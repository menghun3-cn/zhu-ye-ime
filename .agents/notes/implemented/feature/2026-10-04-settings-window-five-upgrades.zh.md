# Agent Note: 设置窗口五项增强（T-088）

Status: implemented

English | [中文](2026-10-04-settings-window-five-upgrades.zh.md)

## Problem

FR-048（M13）交付设置窗口五项增强：用户词表导入导出、符号集扩充、主题文件
（自定义主题）、启动时异步检查一次、通讯录 vcf 界面化导入（D-53，用户
2026-10-02 确认；方案 §14.5）。约束：设置窗口与 IME 进程零联网（D-44 /
S-4）、在线更新默认关闭（零 spawn、零出站）、`config.json` 格式版本不递增
（S-7）、导入先校验后写（FR-042）、系统高对比度仍然接管手选主题（D-31）。

## Decision

五个子系统，各自独立且带单元测试：

1. **用户词表导入导出**（`zhu_ye_core::user_words_exchange`）：v1 Schema
   `{format:"zhu-ye-user-words", version:1, items:[{pinyin, word, freq}],
   exported_at}`。导出用 GetSaveFileNameW，默认文件名
   `user_words_YYYYMMDD.json`；导入用 GetOpenFileNameW → 整文件解析（任一
   条目错误或 `version > 1` 整体拒绝）→ 按 `(pinyin, word)` 合并取 freq max →
   原子写盘；失败绝不触碰磁盘文件。合并经由 `UserDictionary::merge_external`
   组合：忽略空词/空拼音/零 freq，既有键保留 `last_used`、新键置 0。磁盘
   `user_words.json`（v1 `FileFormat`）格式不变。
2. **符号集扩充**（D-33）：core 静态表从 FR-028 三组各 9（27 字符）扩到
   **230 字符**（9 个扩展组 `EXTRA_SYMBOL_GROUPS` 203 字符；v 模式快捷符号
   不动）。设置面板枚举 `all_panel_groups`；SendInput + UNICODE 上屏路径不变。
3. **主题文件/自定义主题**：`themes\*.json` Schema v1 分两节——`candidate`
   （7 键 = 候选窗配色）与 `settings`（15 键 = 设置窗配色）。两窗口解析同一
   `config.theme` 的 `Custom(name)` → 同一文件（S-2）。名称安全：
   `is_safe_theme_name` 只许 `[A-Za-z0-9_-]`（防目录逃逸）；文件缺失/JSON
   损坏/`version > 1` → 拒绝（返回 `None`）；未知键忽略；缺失或非法颜色键 →
   按窗口回退（设置窗 → 系统当前深浅预设 `settings_theme(kind)`；候选窗 →
   浅色基底）。设置窗主题子视图按显示名列出 `themes\*.json`；选中即写
   `config.theme` 并在会话内立即应用（`SettingsTheme::with_theme_file`
   报告覆盖键数）；损坏行给提示并保持当前主题。系统高对比（D-31）接管并
   禁用手选。不做主题在线分发。
4. **启动时异步检查一次**：updater 新增 `check-once` 子命令——全程静默、
   无 stdout。门：`online_update` 开 ∧ `last_check` 距今超 7 天
   （`check_due`）。到期时拉取 manifest、找过期的可分发包、原子写
   `update_status.json`（tmp + rename）、回写 `config.last_check`（S-8
   读-改-写）。IME 侧：`DllGetClassObject` 每宿主进程至多一次 spawn
   `zhu-ye-updater.exe check-once`（static `AtomicBool` 开关），detached +
   `CREATE_NO_WINDOW` + `BELOW_NORMAL_PRIORITY_CLASS`（<10ms）；
   `online_update` 关 → 零 spawn。设置窗关于页先展示手动检查结果，其次读
   `update_status.json`，都无 → "尚未执行检查。"。
5. **通讯录 vcf 界面化导入**：无独立通讯录词表持久化——导入把所选 .vcf
   复制进数据目录 `contacts\`（重名自动加 `(N)` 防覆盖）并把路径追加进
   `config.contact_vcards`（规范化路径去重、保存前重读配置，S-8）。子视图
   列出当前路径；导入前弹窗确认解析出的 N 条联系人；解析复用既有
   `parse_vcard`。

## Alternatives considered

- **独立通讯录词表持久化**：否决——FR-036 数据层已从 `contact_vcards`
   供词给引擎；再造一份持久化只会重复状态并漂移。
- **设置窗口作启动检查宿主**：否决——设置进程不常驻（FR-039 关窗即退）；
   检查归属 updater 进程（D-44：窗口/引擎永不联网）。
- **每次引擎创建都 spawn**：否决——按应用/进程反复拉起；每宿主进程一次 +
   配置门即可。
- **把主题全部颜色内联进 `config.json`（键扩张）**：否决——S-7 保持配置
   格式版本不递增；`config.theme` 只存主题*名*，颜色在文件里。
- **主题在线分发**：否决（既有 §17.3 边界）。
- **主题严格解析（任一未知/缺失键即拒）**：否决——宽松解析（未知键忽略、
   缺键回退预设）才能让旧主题文件在配色扩增后继续可用；严格性由版本字段承担。

## Consequences

- `config.json` 新增可选 `contact_vcards` 数组（缺省 = 空列表），`theme`
   可存自定义主题名；格式版本不递增（S-7）。
- `update_status.json` 为数据目录新文件：
  `format`/`version`/`last_checked`/`available`/`outdated_packs`/`latest_version`/`error`。
- `CandidateWindow` 增加可选自定义 `ThemeFile`；`resolve_with_custom` 保持
  浅色基底与高对比接管。
- 无新增联网路径、零新增运行时依赖；设置窗新增三个子视图及
  `--user-words`/`--contacts`/`--themes` 截图模式供宿主取证。

## Verification

已交付（2026-10-04）：core 测试——user_words_exchange 5、theme_file 7、
update_status 6、time 5、symbols 预算——settings 101、ime 170（含
`theme_with_candidate` 叠加测试）；workspace 15 组全绿；fmt/clippy
`-D warnings` 零告警。`check-once` 宿主实测：关闭/未到期 → exit 0 零网络
零写盘；到期 → 写 status 文件并回写 `last_check`。宿主截图
`data/artifacts/t088-shots`：三子视图截图两两互异（像素差分 ≥890 采样点），
关于页更新子视图在 status 文件存在时正常渲染。VM 交互项（真实 TSF 装配、
抓包 0 出站、冷启动计时）在 VM 可达前显式挂起。

相关记录（部分重叠均保留活跃并交叉链接）：[候选窗浅色默认与空面板（浅色
基底与高对比优先级不变；自定义主题只叠加颜色）](../../implemented/feature/2026-09-25-candidate-window-light-default-and-empty-panel.zh.md)、
[设置窗关于/更新页（现亦读 update_status.json）](../../implemented/feature/2026-10-02-settings-about-and-update-page.zh.md)、
[用户词典持久化（user_words.json 格式不变；交换格式并列新增）](../../implemented/feature/2026-09-19-user-dict-persistence.zh.md)、
[通讯录场景九（FR-036 数据层；界面化导入落在配置登记上）](../../implemented/feature/2026-10-02-contacts-scenario9.zh.md)、
[格式/符号/emoji 候选（FR-028 v 模式未触动）](../../implemented/feature/2026-09-30-format-symbol-emoji-candidates.zh.md)、
[词典更新信任链（check-once 只报告过期包；信任链本体未变）](../../implemented/process/2026-09-29-dictionary-update-trust-chain.zh.md)。
