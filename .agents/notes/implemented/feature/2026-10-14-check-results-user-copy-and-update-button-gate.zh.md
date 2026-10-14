# Agent Note: 检查更新结果面向用户；更新按钮仅在发现更新后可点（T-161）

Status: implemented

## Problem

2026-10-14 用户 verbatim 需求：

> 点击检查更新时，输入的信息不要包含 清单地址、清单、签名校验、可用更新（未应用...）
> 等信息，如果版本是最新的，提示已经是最新版本，如果不是最新版本，提示可用版本及
> 版本更新的日志，有最新版本后，更新按钮才可以点击

此前设置窗口「检查更新」结果区**原样粘贴**更新器的人类诊断输出——清单地址、清单
摘要（schema/发布时间/包数）、签名校验结果、`可用更新: [...]（未应用…）` 包列表。
用户要的是结论性的判断，不是内部诊断；且「应用更新」按钮在没有发现更新时应当置灰。

## Decision

1. **`check --json` 结构化契约**：`zhu-ye-updater check` 新增 `--json`，输出一个
   `UpdateStatus` 对象——与 `check-once` 写 `update_status.json` 完全相同的数据类型
   与语义（失败时以 `error` 字段表达、进程仍以 0 退出；JSON 输出在任何网络/验签失败
   下都成立，契约是全的）。人类诊断行保留给命令行（不带 `--json` 的 `check` 行为
   不变），收集在 `CheckInfo.human_lines`，仅人类模式逐行打印；`online_update=false`
   离线态同样输出 JSON（`failed` + 明确错误）。
2. **设置窗口检查改走 `run_check`**：Check 任务调用 `updater check --json` 并解析为
   `UpdateStatus`（`zhu_ye_core::parse_update_status`，与状态文件共用）；结果区按
   结构化数据渲染而非原始文本——旧文本路径只剩「应用更新」（下载/应用过程反馈）。
3. **面向用户渲染（`summary_lines`）**：四个状态、一个共享纯函数，手动检查路径与
   启动异步状态文件路径（`update_status_lines` 改为 `summary_lines` + 检查时间行）
   两处共用：
   - 失败 → `检查失败：{error}`；
   - 发现输入法新版本 → `发现输入法新版本 vX（当前 vY）` + `更新日志：` + 日志行，
     词库有过期时加「同时有 N 个词库文件可更新，点「应用更新」更新。」，并保留
     「主程序升级请到官网…」引导行；
   - 已最新 → `输入法已经是最新版本（当前 vX）。`；
   - 仅词库可更新 → 上行 + `词库有更新内容（N 个），点「应用更新」更新。`
   包 id 与 `latest_version` 日期不再展示（词库只按数量一句话交代）——词库维度仍可
   操作，但不泄漏内部信息。
4. **「应用更新」按钮加更新门槛**：`can_apply_updates` = 原 `can_run_updater` 且
   `check_has_update`（优先最近一次检查结果，其次本机状态文件）；没有发现更新 →
   按钮置灰（禁用绘制 + 命中不响应），与其上方的「已经是最新版本」提示自洽。
5. **部署耦合**：新设置配旧更新器会解析失败并在结果区显示明确的解析错误——settings
   与 updater 必须成对更新到安装位（T-161 部署重建两者；updater 内置公钥从部署位
   二进制提取，是公钥——`832dc4a5…67cce`——不是秘密）。

## Alternatives considered

1. **设置窗口解析人类输出**（扫描「清单地址」「可用更新」等关键词）：契约落在本地化
   散文上，便宜但脆弱；否决——结构化契约就是一次序列化的事，且 `UpdateStatus` 已存在。
2. **手动检查改走 `check-once`**（复用状态文件）：`check-once` 在 7 天间隔内提前
   退出，「手动检查」可能静默跳过联网；否决。
3. **保留原始输出、只隐藏若干行**：视图层过滤等于重新实现解析，且无法稳健驱动按钮
   门槛；否决。

## 后果

- `zhu-ye-updater`：`check [--json]`；`CheckInfo` 收集 manifest/应用元数据/过期包 +
   人类行；`to_status()` 与 `check-once` 的状态合成口径一致。
- `zhu-ye-settings`：`UpdateOutcome` 改 `OutcomePayload::{Summary, Text}` 双载荷；
   更新器桥新增 `run_check`/`parse_check_output`；新增 `summary_lines`（手动+启动
   渲染共用）、`can_apply_updates`/`check_has_update`/`status_has_update`。
- 测试：updater JSON roundtrip；settings `parse_check_output`（合法+非法）、
   `summary_lines`（四个状态 + 「不含内部词」断言）、`update_status_lines`、
   `status_has_update`（临时目录状态文件）。settings 128 / updater 3 全绿。
- 真实 feed 验证：`check --json` → `available=true`、`outdated_packs=[it,med,slang]`、
   无 error（真公钥验签通过）。
- 后续维护者须知：「应用更新」门槛的真值 = `UpdateStatus` 的 `app_version`/
   `outdated_packs` 字段；用户可见文案的唯一副本是共享的 `summary_lines`。

## 相关笔记

- [在线更新默认开启；输入法版本与更新日志经签名 app.json 提供（T-154）](../../implemented/feature/2026-10-13-online-update-default-on-app-meta.zh.md)
  —— 本任务的契约与文案建立在其之上；两者决策均仍有效。
- [设置「关于与更新」页（T-143/T-150）](../../implemented/feature/2026-10-02-settings-about-and-update-page.zh.md)
  —— 页面结构与 D-44 单一联网组件边界，本任务保持之。
- 验收：docs/验收标准.md §22（在线更新）；任务清单 todos-list T-161。
