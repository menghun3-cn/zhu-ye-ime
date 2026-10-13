# Agent Note: 在线更新默认开启；输入法版本与更新日志经签名 app.json 提供（T-154）

Status: implemented

## Problem

2026-10-13 用户三条 verbatim 需求：

1. 「在线更新，去掉文字说明中的（P-03），改为默认开启」——更新文案不再出现
   （P-03），且功能**默认开启**；
2. 「自动检查更新，如果有新版本则显示最新的版本及更新日志」——启动时自动检查
   发现有新版本时，展示最新输入法版本号与更新日志；
3. 「联机更新的文案不要出现（D-44）之类的说明」——用户可见文案不得出现内部
   决策编号（如 D-44）。

澄清确认（ask_user_question）：检查对象 = **IME 应用版本 + 更新日志**（非词典包）；
展示位置 = 设置窗口「关于与更新」页常驻展示；老配置升级 = **缺失 `online_update`
键一律按开启**（显式 `false` 保持关闭）。

本任务反转 2026-09-29 信任链 note 以来沿用的 P-03「默认关闭」决策，并为发布
流水线新增一个签名元数据文件。

## Decision

- **默认开启反转**：`ConfigFile.online_update` 改 `#[serde(default =
  "default_online_update")]` 返回 `true`，`Default` impl 同步——旧配置缺键
  按开启（升级路径），显式 `false` 仍尊重，更新器「网络请求」门禁（false ⇒
  零 spawn / 零网络）不变。开关 UI 描述不再写「默认关闭，需显式开启」。
- **输入法应用版本走独立签名 `app.json`**——绝不作为 `manifest.json` 的新字段。
  manifest 签名是对**规范化** JSON 计算（`serde_json::to_vec`：字段序 = 结构体
  声明序、排除 signature 字段）；若给 manifest 加字段，旧版二进制解析时 serde
  丢弃未知字段、重新规范化与发布方不一致 → 验签失败。全新资产、旧版更新器
  根本不读取 → 天然向后兼容。`app.json` 与 `manifest.json` 同目录，复用 P-04
  信任模型：ed25519 签名规范化 JSON，客户端**内置公钥**验签。
- **`AppMeta::verify` 只用传入的 trusted 公钥**（`&VerifyingKey`），绝不使用
  文件内嵌公钥验签（内嵌公钥仅审计）。初版照抄 manifest 的 `verify_signature`
  逻辑用文件内嵌公钥验——安全缺陷，被「错公钥」单测当场抓回。
- **版本比较双向**：
  `app_update_available(app, core) = version_at_least(app, core) && !version_at_least(core, app)`
  —— 仅当应用版本严格更高才报「有更新」（每段取前导数字、忽略 pre-release
  后缀，与 `min_engine_version` 同口径）。单向比较会把语义写反。
- **`UpdateStatus` 保持 `version: 1`**，新增两个可选字段（`app_version`、
  `release_notes`，`#[serde(default, skip_serializing_if)]`）与新构造器
  `available_app(last_checked, app_version, release_notes, outdated_packs, latest_version)`；
  旧 v1 文件缺失字段读作 `None`。check-once 在应用有新版本时合成它（词典包
  信息一并携带），关于页两种更新同时可见。
- **降级而非失败**：`fetch_app_meta` 中 HTTP 拉取不到（404 / 无资产 / 网络）
  返回 `Ok(None)`（「发布源未提供应用版本信息」）；解析/验签失败返回 `Err`
  （一行提示）——两者都不阻断词典包检查与应用，check-once 不因 app.json
  写 `failed`。
- **结果区 3 → 6 行**（`LOGICAL_RESULT_ROWS`），`update_status_lines` 输出
  「发现输入法新版本 vX（当前 vY）」+「更新日志：」+ 逐条日志行；词典包信息
  同时存在时附后。
- **用户可见字符串无内部编号**（P-03 / D-44 / P-12 …）；代码注释保留编号
  可追溯。即 dsh-prose-standard 规则落到 UI 文案。
- **发布工具链**：`zhu-ye-dict` 新增 `build-app` / `sign-app` / `verify-app`；
  `assemble-release.ps1` 生成 `app.json`（app_version = 根 Cargo.toml 版本，
  release_notes = CHANGELOG 最新段经 `plain_text_notes` 纯文本化——去
  Markdown 标题/链接/反引号）、签名并纳入上传清单与 SHA256SUMS.txt；
  `verify-release-e2e.ps1` 增加两步：app.json 良性展示与篡改降级。
  CI（`release.yml`）把 `app.json` 一并上传主仓 Release 与更新源镜像 feed
  （T-155），发布端闭环补齐——真实发版即可端到端携带应用版本信息。

## Alternatives considered

- **版本号加进 `manifest.json`**：破坏旧版验签（规范化漂移）——否决；独立
  资产是唯一向后兼容路径。
- **复用 manifest 的 `version` 当应用版本**：词典包版本是日期型发布串、与
  引擎版本解耦——无法从中推导输入法版本——否决。
- **只改文案、默认仍关**：用户明确要求反转默认值——否决。
- **app.json 失败按检查失败处理（写 `failed`）**：可选展示增强不得阻断包更新
  或误置状态——否决，降级。
- **用文件内嵌公钥验签**：被篡改的文件可自证清白——否决；只用内置公钥，内嵌
  公钥是审计数据。

## Consequences

- 已发布的旧版更新器完全不读 `app.json`——无签名破裂、无需服务端兼容补丁。
- 每次发布资产变为四件：zip、packs、签名 `manifest.json`、签名 `app.json`；
  e2e 脚本覆盖两份签名（第 4-6 步）。
- settings / updater / core 三侧都变；workspace 全绿（core 347、settings 118、
  updater 2、dict 87），新测试锁定：默认开启反转、缺键升级、双向比较、
  v1 兼容状态、发布说明纯文本化。
- 在发布的更新源有 `app.json` 之前，关于页显示「已是最新版本」或「（发布源未提供
  应用版本信息）」——正确降级，不是缺陷。
- 本机无发布私钥：完整签名仅以 `-SkipSign` dry-run 验证生成逻辑；「发现新版本 +
  更新日志」需等真实 v0.1.3 发布才能端到端目视（已记入验收 §22）。

## Related

- 反转 [2026-09-29-dictionary-update-trust-chain](../../implemented/architecture/2026-09-29-dictionary-update-trust-chain.zh.md)
  的默认关闭决策（P-03 引入点）；信任链机制本体（P-04）不变。
- 部分取代 [2026-10-02-settings-about-and-update-page](../../implemented/feature/2026-10-02-settings-about-and-update-page.zh.md)
  （界面门禁保留、默认值反转）与
  [2026-10-04-settings-window-five-upgrades](../../implemented/feature/2026-10-04-settings-window-five-upgrades.zh.md)
  （check-once 门禁保留、默认值反转）。
- 相邻更新路径改动：[2026-10-13-updater-spawn-no-console-window](../../implemented/bug-fix/2026-10-13-updater-spawn-no-console-window.zh.md)
  （T-153，同一 check/apply 子进程管线）。
- 验收：docs/验收标准.md §22；任务清单 todos-list T-154。
