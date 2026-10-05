# Agent Note：语言栏按钮左键点击切换中英模式（T-100，T-046 扩展）

Status: proposed

English | [中文](2026-10-05-langbar-click-toggle.zh.md) | 中英镜像

## Problem

语言栏图标（"中"/"英"，implemented note 2026-09-28-language-bar-mode-icon）已随引擎模式
切换，但按钮 `OnClick` 是空实现——实现当时引擎状态为 `Rc<Mutex<EngineState>>`（非 `Send`），
而 ctfmon 可能经任意 RPC 线程调用项目的 COM 方法，跨线程进入引擎会破坏既有线程模型假设。
用户指名："语言栏按钮点击切换中英模式"（已接受 11 项清单批二第 1 项）。

## Proposal

1. **左键点击切换模式**。`TF_LBI_CLK_LEFT`（1）时运行注入的点击处理器，返回值写入按钮
   （经 advise 的 `ITfLangBarItemSink::OnUpdate(TF_LBI_ICON)` 刷新图标）。其它点击
   （如右击菜单）保持原行为。
2. **引擎共享状态迁移为 `Arc<SharedEngine>`**。`SharedEngine(Mutex<EngineState>)` 经
   `unsafe impl Send + Sync` 可跨线程迁移，三条安全论证（见 tsf.rs 类型注释）：
   ① 所有非 `Send` 字段（COM 接口、候选窗裸指针）只在持有锁期间被访问，移动锁拥有者
   不移动锁内数据所有权；② 最后一个强引用只在键盘/激活线程释放——语言栏 `OnClick`
   仅经 `upgrade` 产生临时强引用，`self.state` 在按钮存活期间恒持有状态——Drop 线程
   与 Rc 时代一致（候选窗 `Drop` 里的 `DestroyWindow` 从不在跨线程执行）；③
   `IUnknown::Release` 线程无关。对后续工作的启示：字段全部锁内访问的 `Mutex<T>`
   需要跨线程迁移时，优先用"newtype + `unsafe impl`"包装，而不是逐字段补 `Send`
   （孤儿规则）或散布零散 unsafe impl。
3. **无引用环**。`LangBarModeButton` 增加
   `click_handler: Box<dyn Fn() -> Option<InputMode> + Send + Sync>`，只捕获
   `Weak<SharedEngine>`；`EngineState.lang_bar → LangBarModeButton → Weak` 不成环。
   `upgrade` 失败（引擎不可达）时点击安全忽略（返回 `None`，模式不变）。
4. **语义与 Shift 切换一致**。处理器镜像 `sync_engine::ToggleMode`：锁内 `toggle_mode()`、
   锁外通知图标；组合中的内容保留（`InputEngine` 既有语义）。

## Alternatives considered

- **整体 `Arc<Mutex<EngineState>>`（EngineState: Send）**：需要在 windows 接口
  （`ITfKeystrokeMgr`、`ITfComposition` 等）上实现 `Send`，孤儿规则禁止本地 impl →
  落选；newtype 承载锁纪律论证。
- **跨线程消息投递（PostMessage 到候选窗线程）**：引入消息泵依赖与排队时序，相比
  锁内切换无收益 → 落选。
- **模式原子量双写（引擎与按钮各持 `AtomicU8`）**：双事实源易漂移 → 落选；按钮模式
  只是展示缓存，引擎锁内为权威，两条写路径（Shift 同步、`OnClick`）都会收敛。

## Acceptance criteria

- Win11 桌面语言栏开启（设置 → 个性化 → 任务栏 → 输入法指示器 → 使用桌面语言栏）：
  出现"中"按钮；左键点击立即变"英"；再点回"中"。
- 切换联动真实行为：中文模式字母进拼音组合；英文模式字母直通上屏。
- 组合中点击切换：拼音组合内容保留（与 Shift 切换一致）。
- 停用输入法（切走再切回）后按钮与引擎模式一致，无残留状态。
- 快速/连续点击：无崩溃、无图标错乱（切换在引擎锁内串行化）。

## Risks

- 按钮 `mode` 与引擎 `mode` 仍然是两份值；按钮只是展示缓存，并发切换后读取方可能
  短暂看到陈旧图标——仅视觉，行为权威仍在引擎锁内。
- `EngineState` 新增字段必须复核"锁内访问"纪律（`SharedEngine` 论证的依赖前提）；
  tsf.rs 的注释即审计锚点。
- ctfmon RPC 线程与键事件 sink 由同一把锁串行化，但 `std::sync::Mutex` 不可重入：
  处理器不得在持锁期间回调 `sync_engine`——实现把切换（持锁）与图标通知（释放锁）
  严格分离。
