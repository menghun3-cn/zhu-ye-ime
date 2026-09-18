# Agent Note: TSF composition and key handling for M1

Status: implemented

[English](2026-09-18-tsf-composition-and-key-events.md) | 中文

## 问题

已注册的 TSF 文本服务只有在按键事件能变成宿主文档中的可见文本之后才可用。没有按键过滤、组合与提交，输入法虽然出现在 Windows 中，敲击却不会产生任何内容；没有统一的引擎契约，TSF 适配层与将来的候选窗 UI 还容易各自漂移。

## 决策

`TextService` 实现 `ITfTextInputProcessorEx`、`ITfKeyEventSink` 与 `ITfCompositionSink`。`Activate` 时从线程管理器解析 `ITfSource` 并注册按键事件 sink，持有 sink cookie；`Deactivate` 结束当前组合、清空引擎状态并取消订阅。允许空线程管理器，使生命周期单元测试与自检无需 TSF 宿主也能运行。

按键映射为小型 `KeyAction` 集合：a-z 进入组合，Backspace 删除组合末尾；空格、回车、Esc 与 1-9 仅在已有组合时提交或取消。`OnTestKeyDown` 报告输入法是否要吃键；`OnKeyDown` 请求使用 `TF_ES_SYNC | TF_ES_READWRITE` 的同步读写编辑会话。若编辑会话无法获得，仍推进引擎状态，避免后续按键基于漂移状态继续。

在编辑会话内，`ITfEditSession::DoEditSession` 执行一次 `apply_action` 回调。字母/Backspace 更新经 `ITfInsertAtSelection`、`ITfContextComposition::StartComposition` 与 `ITfRange::SetText` 完成；空格、回车、Esc 与数字通过 `EndComposition` 结束组合，或在没有组合时直接插入提交文本。TSF 文本写入后再同步引擎。

输入行为本体位于 `crates/zhu-ye-ime/src/input.rs`，是带 M1 演示词典的纯 Rust `InputEngine`，候选生成、排序、选择与预览方法都可在无 Windows 环境下单测。共享运行时状态采用 `Rc<Mutex<EngineState>>` 而非 `Arc`：TSF apartment 回调始终在同一线程，且保存的 COM 接口不保证 `Send + Sync`。每个编辑会话回调对象携带一个 `Box<dyn FnOnce>`，只执行一次。

目前尚未实现显示属性与候选窗。空格上屏第一候选或拼音原文，回车提交拼音，Esc 取消。T-012、T-013 在此接口之上继续。

## 曾考虑的替代方案

**组合状态完全留在 TSF 适配层。** 落选：候选逻辑会被困在 COM 胶水代码里且无法在开发主机测试；纯引擎才是后续词典与排序工作的可单测核心。

**先改引擎再写 TSF，失败时回滚。** 落选：TSF 文本变更无法原子回滚，先写后同步可避免引擎与宿主永久错位。

**每次更新都重建组合。** 落选：保存 `ITfComposition` 并复用 `GetRange`/`SetText` 更简单，也不会造成焦点意外。

**使用 `Arc<Mutex<EngineState>>` 并强制 `Send + Sync`。**
落选：TSF 运行时 COM 对象不保证 `Send + Sync`；`Rc<Mutex<_>>` 匹配单 apartment 线程，并保持 clippy `-D warnings` 门禁通过。

**请求异步编辑会话。** M1 延期：同步会话保持单一按键流水线、避免交错；若宿主在长操作上阻塞，再引入异步。

## 后果

字母、Backspace、空格、回车、Esc 与 1-9 现在能产生 TSF 组合或提交文本，M1 演示词典会让 `nihao` 在组合路径中变成 `你好`。引擎与 TSF 适配层共 20 项单元测试通过，`cargo clippy --workspace --all-targets -- -D warnings` 通过，release DLL 导出校验通过。

T-011 保持"进行中"，直到当前构建安装到真实 Windows 测试机（正在准备的虚拟机）并在记事本中验证输入为止。早先注册笔记中"M1 无按键处理"的描述已不匹配当前构建；本笔记代表当前 TSF 适配层的事实。候选窗渲染与完整键位交互（Shift/Tab/翻页）延后到 T-012/T-013。
