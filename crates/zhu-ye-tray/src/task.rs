//! 工具箱面板任务的**串行队列线程**（T-169，S-35）。
//!
//! 面板（`ZhuYeSettingsPanel`）是带自己消息循环的非激活窗口：`PanelWindow::open`
//! 只建窗，`Drop` 才 `DestroyWindow`。它不能建在托盘主线程上——主线程要跑托盘
//! 消息循环与 `TrackPopupMenu` 模态菜单；也不能每个请求一个临时线程——两次
//! 快速 toggle 会让面板在销毁中途被再次操作。这里用**一个长驻工作线程 + 串行
//! 队列**：面板的创建、关闭、消息泵都在同一线程上顺序执行，托盘主线程只投递
//! 任务、只读回报。
//!
//! 本模块不直接依赖 `windows` crate 的窗口消息类型，跨线程约定只有：
//! 托盘窗口句柄（`isize`）、两条自定义消息号（见 [`start`]）、以及投递的
//! [`PanelAction`]——全部以裸值传递，Send 无争议。

use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Mutex;
use std::time::Duration;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;

use zhu_ye_settings::panel::PanelView;
use zhu_ye_settings::panel_window::{self, PanelWindow, PANEL_CLASS_NAME};

use crate::{PANEL_ESCAPE_MS, WM_TASK_DONE};

/// 投递给面板线程的动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PanelAction {
    /// 打开面板（已开则忽略）；按上次页签/位置记忆创建。
    Open,
    /// 关闭面板（未开则忽略）。
    Close,
}

/// 工作线程收到的消息。
#[derive(Debug, Clone, Copy)]
pub(crate) enum TaskMessage {
    /// 执行一个面板动作。
    Do(PanelAction),
    /// 立即结束线程（面板随持有销毁）。
    Stop,
}

/// 队列：进程内只有一个托盘，`OnceLock` 风格的全局 `Mutex<Option<Sender>>` 足够。
static QUEUE: Mutex<Option<Sender<TaskMessage>>> = Mutex::new(None);
/// 待处理动作（工作线程取走前的工作清单，只为可观测性保留）。
static PENDING: Mutex<VecDeque<PanelAction>> = Mutex::new(VecDeque::new());

/// 投递一个面板动作；任务队列未启动（线程启动失败）时返回 `false`，由调用方降级。
pub(crate) fn dispatch(action: PanelAction) -> bool {
    let guard = match QUEUE.lock() {
        Ok(guard) => guard,
        Err(_) => return false,
    };
    let Some(sender) = guard.as_ref() else {
        return false;
    };
    if let Ok(mut pending) = PENDING.lock() {
        pending.push_back(action);
    }
    sender.send(TaskMessage::Do(action)).is_ok()
}

/// 退出前收尾：让工作线程关闭面板并结束自身（重复调用无副作用）。
pub(crate) fn request_shutdown() {
    let sender = QUEUE.lock().ok().and_then(|guard| guard.clone());
    if let Some(sender) = sender {
        let _ = sender.send(TaskMessage::Stop);
    }
}

/// 启动串行队列线程。`tray_window` 是回报目标（任务完成后 `PostMessage(WM_TASK_DONE)`）。
///
/// 回报参数：`lparam.0` = 面板当前是否打开（1/0），`wparam` 未使用。
/// 窗口句柄以 `isize` 跨线程传递（`HWND` 在 windows crate 里是 `*mut c_void`，不满足 `Send`）。
pub(crate) fn start(tray_window: HWND) -> Result<(), String> {
    let (sender, receiver) = channel();
    {
        let mut guard = QUEUE.lock().map_err(|_| "队列锁中毒".to_owned())?;
        *guard = Some(sender);
    }
    let target = tray_window.0 as isize;
    std::thread::Builder::new()
        .name("zhu-ye-tray-panel".to_owned())
        .spawn(move || run(receiver, HWND(target as *mut core::ffi::c_void)))
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// 工作线程主体：串行执行面板动作，并在面板打开期间泵它自己的消息循环。
fn run(receiver: Receiver<TaskMessage>, tray_window: HWND) {
    let mut panel: Option<PanelWindow> = None;
    loop {
        // 面板打开时用 15ms 超时轮询：既处理面板消息，也及时接住新任务。
        let deadline = Duration::from_millis(15);
        let message = if panel.is_some() {
            pump_panel_messages(&mut panel);
            match receiver.recv_timeout(deadline) {
                Ok(message) => Some(message),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        } else {
            match receiver.recv() {
                Ok(message) => Some(message),
                Err(_) => break,
            }
        };
        let Some(message) = message else {
            report(tray_window, panel.is_some());
            continue;
        };
        match message {
            TaskMessage::Stop => {
                drop(panel.take()); // Drop 销毁窗口后结束线程
                break;
            }
            TaskMessage::Do(PanelAction::Open) => {
                take_pending(PanelAction::Open);
                if panel.is_none() {
                    panel = open_panel();
                }
            }
            TaskMessage::Do(PanelAction::Close) => {
                take_pending(PanelAction::Close);
                panel = None;
            }
        }
        report(tray_window, panel.is_some());
    }
    if let Ok(mut pending) = PENDING.lock() {
        pending.clear();
    }
}

/// 从待处理清单里去掉刚执行的动作（仅观测用）。
fn take_pending(action: PanelAction) {
    if let Ok(mut pending) = PENDING.lock() {
        if let Some(index) = pending.iter().position(|item| *item == action) {
            pending.remove(index);
        }
    }
}

/// 泵面板窗口的待处理消息（面板与队列线程同线程，正常分发）。
fn pump_panel_messages(panel: &mut Option<PanelWindow>) {
    let Some(window) = panel.as_ref().map(PanelWindow::hwnd) else {
        return;
    };
    if window.is_invalid() {
        *panel = None;
        return;
    }
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
    };
    let mut message = MSG::default();
    while unsafe { PeekMessageW(&mut message, Some(window), 0, 0, PM_REMOVE) }.as_bool() {
        let _ = unsafe { TranslateMessage(&message) };
        unsafe { DispatchMessageW(&message) };
    }
    // 面板自己关掉自己（Esc 窗口期内按 Esc）→ 句柄失效，丢弃持有。
    let alive =
        unsafe { windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(window)) }.as_bool();
    if !alive {
        *panel = None;
    }
}

/// 创建面板（全局单面板互斥：其它进程已开则不重复创建）。
fn open_panel() -> Option<PanelWindow> {
    if panel_exists() {
        return None;
    }
    let kind = panel_window::remembered_kind();
    match PanelWindow::open(PanelView::new(kind), None) {
        Ok(mut panel) => {
            // 托盘双击唤出：窗口期内 Esc 可关闭（T-169）。
            panel.listen_escape(PANEL_ESCAPE_MS);
            Some(panel)
        }
        Err(error) => {
            crate::log_panel_hotkey(&format!("[error] 呼出工具箱面板失败：{error}\n"));
            None
        }
    }
}

/// 面板窗口类是否已被创建（任意进程；S-31 全局单面板）。
fn panel_exists() -> bool {
    let class: Vec<u16> = PANEL_CLASS_NAME.encode_utf16().collect();
    match unsafe {
        windows::Win32::UI::WindowsAndMessaging::FindWindowW(
            windows::core::PCWSTR(class.as_ptr()),
            None,
        )
    } {
        Ok(existing) => !existing.is_invalid(),
        Err(_) => false,
    }
}

/// 回报面板状态给托盘窗口（失败静默：托盘可能已在退出）。
fn report(tray_window: HWND, open: bool) {
    let open_flag = if open { LPARAM(1) } else { LPARAM(0) };
    unsafe {
        let _ = PostMessageW(Some(tray_window), WM_TASK_DONE, WPARAM(0), open_flag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 队列投递入队、动作去重、Stop 收尾都是纯内存行为，不建窗口也不起线程。
    #[test]
    fn 待处理清单先进先出且按动作去除() {
        PENDING.lock().unwrap().clear();
        PENDING.lock().unwrap().push_back(PanelAction::Open);
        PENDING.lock().unwrap().push_back(PanelAction::Close);
        take_pending(PanelAction::Open);
        let remaining: Vec<_> = PENDING.lock().unwrap().iter().copied().collect();
        assert_eq!(remaining, vec![PanelAction::Close]);
        take_pending(PanelAction::Close);
        assert!(PENDING.lock().unwrap().is_empty());
        // 去掉不存在的动作不 panic。
        take_pending(PanelAction::Open);
    }

    /// 未启动时投递必须明确失败（调用方据此降级为热键入口）。
    #[test]
    fn 未启动时投递失败() {
        *QUEUE.lock().unwrap() = None;
        assert!(!dispatch(PanelAction::Open));
    }
}
