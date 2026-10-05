# Agent Note：发布收尾批一 —— exe 安装包 / 更新源 / Pages（第十二期）

状态：proposed（待确认）

[English](2026-10-05-release-closure-batch-one.md) | 中文

## 问题

M14 之后仍有三个发布收尾缺口，全部登记在第十期官网需求（D-69 下载形态、P-01 镜像
留待后续、D-68 Pages 真部署后续）与官网路线图"流程收尾中"：

1. **无 exe 安装包**：下载页只有便携 zip（内测期产物），exe 行标注"规划中"；唯一安装
   入口是管理员 PowerShell 运行 `scripts/install.ps1`——可脚本化、可回滚，但不是产品
   形态，也没有原生卸载入口。
2. **更新源从未真正可用**（2026-10-05 T-096 调研实证）：客户端默认 URL
   `https://github.com/menghun3-cn/zhu-ye-ime/releases/latest/download/manifest.json`
   实测 **302 → 404**。根因是 GitHub Releases 语义：`releases/latest` 只解析到最新
   **非 prerelease**；而 CI 发布（`release.yml`，版本含 `-` 自动加 `--prerelease`）带
   prerelease 标记，且实测 `gh api releases/latest` 证明 **GitHub 拒绝把 prerelease 设
   为 latest（即使显式传 `--latest` 也无效）**。latest 因此停在 v0.1.1-alpha（当年手工
   发布未标记，资产只有 zip、**无 manifest.json**）；v0.1.2-alpha 六项资产齐备却被
   latest 语义排除。更新器把该 URL 编译期写死（`crates/zhu-ye-updater/src/main.rs:30-31`），
   所以已交付客户端永远拿不到 manifest。推论：**只要保持 `-alpha`（prerelease）发布
   节奏，基于 latest 的 URL 就永远不可用**，直到出现非 prerelease 正式版。
3. **Pages 从未部署成功**：`pages.yml` 已就绪、仓库 Pages 配置为 `build_type=workflow`，
   但从无成功部署（`status=null`）；官网只能在仓库内浏览。

## 方案（推荐口径）

第十二期建议范围（D-74~D-78，**待用户确认**；批一 = 三个收尾项，FR-061/062/063）：

- **D-74 exe 技术栈：Inno Setup**。`ai-zhu-ye-ime-setup-<ver>.exe` 单文件 x64、UAC 清单、
  自带卸载器。exe 是既有脚本的**产品化外壳**：载荷与便携 zip 同一 package staging
  （bin/scripts/packs/docs）释放到 `{tmp}\package`，安装步骤调用 `install.ps1`
  （`-SkipBuild -PackageRoot`）——版本化 DLL 复制、PE 导出校验、TSF 注册回滚、预置
  领域包、开始菜单快捷方式、旧 DLL 延迟清理全部原样复用，**零行为改写、无第二个注册
  实现**。卸载走卸载语义（`Remove-TsfRegistration` + 目录/残留清理），显式不动
  `%APPDATA%\ai-zhu-ye-ime` 用户数据。
- **D-75 无 Authenticode 签名**：接受 SmartScreen"更多信息 → 仍要运行"步骤；
  `build-setup.ps1 -SignCommand` 预留签名位（有证书即插）。
- **D-76 更新源 = 独立镜像分发通道**：新建 `menghun3-cn/zhu-ye-updates`；release
  workflow 把 `manifest.json + it/med/slang.zyct`（约 3MB）原子推送过去；客户端
  `DEFAULT_MANIFEST_URL` 改到该通道的稳定 URL（无 prerelease 仓库的 Releases-latest，
  或其 Pages URL——M15-D 最终二选一）。这绕开 latest 缺陷、兑现 P-01"镜像"，且让
  官网与更新源**解耦**——官网重部署永远不会覆盖更新源。
- **D-77 `online_update` 保持默认关闭（P-03 重申）**：开通 = "开启后链路可用"，不是
  "默认开启"；关闭时零网络仍是可审计的不变项。
- **D-78 保持 `vX.Y.Z-alpha` 发布节奏**：镜像不再依赖 latest 语义，prerelease 标记
  不再阻塞更新源；非 prerelease 正式版照常等定期的正式发布时刻。
- **FR-063 Pages 部署** = 触发现有 `pages.yml` → 线上验证脚本
  （`scripts/verify-site-live.ps1`）覆盖六页/AI 入口/下载链；下载页 exe 状态改可用并给
  直链；六页 hero-note/footer 状态行与路线图改"已开通"口径。

## 备选方案（已否决）

**用 Inno Pascal 重写安装逻辑。** 否决：TSF 注册与 DLL 部署的第二实现会重蹈"哪个是
权威实现"的漂移问题（TSF 注册归属 Note 已封口），还要重测已验收的回滚语义；外壳方案
保持单一权威实现。

**更新源内嵌官网（`site/packs/`）一次部署。** 对抗性审查后否决：workflow 模式每次部署
**整体替换站点**，之后任何 `site/**` 变更触发 `pages.yml` 都会静默丢掉更新源，直到下次
发布重新上传——周期性消失的更新源。（备选：教 `pages.yml` 读最新 release 资产合并再
部署——太复杂易错，不划算。）

**`DEFAULT_MANIFEST_URL` 钉到具体版本 tag。** 否决：客户端常量固定在一个版本，永远
发现不了新包。

**等非 prerelease 正式版"顺带开通"。** 否决：把更新源验证绑定到发布日期，且这期间
已交付客户端一直是坏的（现在 404，连验签都到不了）。

**去掉 `release.yml` 的 prerelease 标记。** 否决：让内测构建污染 latest 语义，推翻
"alpha=prerelease"的既有约定。

## 后果

- 更新信任链（manifest.json 内 ed25519 签名 + `ZHU_YE_RELEASE_PUBLIC_KEY` 编译内置）
  **不变**——只挪 URL，信任锚不动；私钥仍只在 Actions secret。
- 更新器保留 `ZHU_YE_MANIFEST_URL` 环境变量覆盖（测试/私有渠道）。
- 分发仓库须保持卫生：受保护分支、无 prerelease 标记的 release（该仓库没有发布
  workflow）、不推送任何密钥材料。
- 安装/卸载保持已验收的回滚行为；覆盖安装幂等；卸载不动用户数据。
- zip 安装路径保留为备选下载（与 exe 载荷同源，哈希一致性入 e2e 断言）。
- release CI 增 1 项资产（setup exe，第 7 项）与 1-2 步（build setup、publish feed）；
  前置 assemble/verify 门禁不变。

## 必须验证（验收 §17）

- 17.1：资产存在；干净机安装 → TSF 注册 → 输入可用；覆盖幂等；卸载无注册/注册表残留
  且保留 `%APPDATA%`；e2e 增"setup exe 静默安装到 temp 目录"参数连通断言；zip/exe
  载荷哈希一致。
- 17.2：默认 URL 拉到 manifest（不再 404）；apply 验签、原子落地、`.bak` 回滚可用；
  篡改包被拒；镜像哈希与 Release 资产一致；`online_update: false` 零网络；
  `ZHU_YE_MANIFEST_URL` 覆盖有效。
- 17.3：`verify-site-live.ps1` 六页 + AI 入口 + 下载链全绿；`check-site-structure.ps1`
  更新后全绿；状态行与路线图线上可见。
- 17.4：eval 零回退（Top1 84.7% / Top3 97.2% / 整句 21.0%，MISS 385）、host-e2e 全绿、
  bench 不劣化、四重门禁全绿。

## 风险

**GitHub 交付抖动（镜像推送、CDN 生效延迟）**：复用既有重试循环 + 线上 e2e 对 CDN
延迟做有界重试。

**安装器与脚本参数漂移破坏外壳设计**：e2e 静默安装步骤在资产上传前断言参数连通。

**分发仓库混入 prerelease 或密钥材料**：受保护分支卫生 + 线上 e2e 卫生检查。

**无签名 exe 被 SmartScreen 拦截**：D-75 接受，下载页写明步骤，SignTool 钩子可逆。

**卸载误删用户数据**：卸载路径显式排除 `%APPDATA%\ai-zhu-ye-ime`，e2e 断言保留。

**镜像成为更新唯一单点**：接受——当前单一镜像，多镜像后置（manifest 无 URL 字段，
后续多镜像设计仍在客户端侧）。

**批一部分依赖用户确认**：范围与三个通道选择（D-74~78 与 Releases-vs-Pages 的 feed
URL）作为推荐草稿推进；文档批次（T-096）以草稿 PR 落地待确认，实现任务
T-097/T-098/T-099 待确认后启动。
