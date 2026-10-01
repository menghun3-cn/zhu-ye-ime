//! 本地最小 TSF 宿主：创建线程管理器/文档/上下文，在编辑会话内复现
//! `InsertTextAtSelection` 的崩溃行为（VM 上 TIP 在该调用处崩，宿主进程被
//! c0000005 击杀；本探针在开发机交互会话来最小化复现，便于快速迭代）。
//! 仅供诊断使用，不属于交付物。

use std::ffi::c_void;
use std::sync::Mutex;

use windows::core::{implement, Interface, Result, PCWSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::TextServices::{
    CLSID_TF_ThreadMgr, ITfComposition, ITfCompositionSink, ITfCompositionSink_Impl, ITfContext,
    ITfContextComposition, ITfContextOwnerCompositionSink, ITfContextOwnerCompositionSink_Impl,
    ITfDocumentMgr, ITfEditSession, ITfEditSession_Impl, ITfInsertAtSelection, ITfThreadMgr,
    INSERT_TEXT_AT_SELECTION_FLAGS, TF_AE_NONE, TF_DEFAULT_SELECTION, TF_ES_READWRITE, TF_ES_SYNC,
    TF_IAS_NOQUERY, TF_IAS_QUERYONLY, TF_SELECTION, TF_SELECTIONSTYLE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 宿主侧文档上下文 sink（CreateContext 所需），方法全部空实现。
#[implement(ITfContextOwnerCompositionSink)]
struct HostSink;

impl ITfContextOwnerCompositionSink_Impl for HostSink_Impl {
    fn OnStartComposition(
        &self,
        _pcomposition: windows_core::Ref<'_, windows::Win32::UI::TextServices::ITfCompositionView>,
    ) -> Result<windows_core::BOOL> {
        Ok(windows_core::BOOL(1))
    }

    fn OnUpdateComposition(
        &self,
        _pcomposition: windows_core::Ref<'_, windows::Win32::UI::TextServices::ITfCompositionView>,
        _prangenew: windows_core::Ref<'_, windows::Win32::UI::TextServices::ITfRange>,
    ) -> Result<()> {
        Ok(())
    }

    fn OnEndComposition(
        &self,
        _pcomposition: windows_core::Ref<'_, windows::Win32::UI::TextServices::ITfCompositionView>,
    ) -> Result<()> {
        Ok(())
    }
}

type EditCallback = Box<dyn FnOnce(u32) -> Result<()>>;

/// 组合回调 sink（StartComposition 所需），方法全部空实现，
/// 模拟真实 TIP 的 ITfCompositionSink。
#[implement(ITfCompositionSink)]
struct CompSink;

impl ITfCompositionSink_Impl for CompSink_Impl {
    fn OnCompositionTerminated(
        &self,
        _ecwrite: u32,
        _pcomposition: windows_core::Ref<'_, ITfComposition>,
    ) -> Result<()> {
        println!("[probe] CompSink OnCompositionTerminated");
        Ok(())
    }
}

#[implement(ITfEditSession)]
struct ProbeSession {
    callback: Mutex<Option<EditCallback>>,
}

impl ITfEditSession_Impl for ProbeSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        println!("[probe] DoEditSession ec={ec}");
        if let Some(callback) = self.callback.lock().unwrap().take() {
            callback(ec)
        } else {
            Ok(())
        }
    }
}

unsafe extern "system" fn probe_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

unsafe fn create_hidden_window() -> HWND {
    let class_name = to_wide("ZhuYeProbeWnd");
    let hinstance = GetModuleHandleW(None).unwrap_or_default();
    let wc = WNDCLASSW {
        lpfnWndProc: Some(probe_wnd_proc),
        hInstance: hinstance.into(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        style: CS_HREDRAW | CS_VREDRAW,
        ..Default::default()
    };
    let _ = RegisterClassW(&wc);
    CreateWindowExW(
        windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE::default(),
        PCWSTR(class_name.as_ptr()),
        PCWSTR(to_wide("probe").as_ptr()),
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        320,
        200,
        None,
        None,
        Some(hinstance.into()),
        None,
    )
    .expect("创建窗口失败")
}

fn main() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .expect("CoInitializeEx 失败");
    }

    let tm: ITfThreadMgr =
        unsafe { CoCreateInstance(&CLSID_TF_ThreadMgr, None, CLSCTX_INPROC_SERVER) }
            .expect("创建 ITfThreadMgr 失败");
    let tid: u32 = unsafe { tm.Activate() }?;
    println!("[probe] Activate tid={tid}");

    let dm: ITfDocumentMgr = unsafe { tm.CreateDocumentMgr() }?;
    let _hwnd = unsafe { create_hidden_window() };
    let sink: ITfContextOwnerCompositionSink = HostSink.into();
    let mut ctx = None;
    let mut ec = 0u32;
    unsafe { dm.CreateContext(tid, 0, &sink, &mut ctx, &mut ec) }?;
    let context = ctx.expect("CreateContext 未返回上下文");
    unsafe { dm.Push(&context)? };
    unsafe { tm.SetFocus(&dm)? };
    println!("[probe] doc/context created, ec={ec}");

    let session: ITfEditSession = ProbeSession {
        callback: Mutex::new(Some(Box::new(move |ec| {
            println!("[probe] callback begin ec={ec}");
            // A: 完全不碰 TSF —— 验证回调自身与 session 生命周期
            println!("[probe] A-only done (no TSF calls)");
            Ok(())
        }))),
    }
    .into();

    println!("[probe] RequestEditSession begin");
    let hr = unsafe { context.RequestEditSession(tid, &session, TF_ES_SYNC | TF_ES_READWRITE) };
    println!("[probe] RequestEditSession hr={hr:?}");
    match hr {
        Ok(hr) if hr.is_err() => println!("[probe] session failed 0x{:08X}", hr.0 as u32),
        _ => {}
    }

    // B: 手写 vtable 调 RequestEditSession（槽 3），对比 ec 是否正常
    println!("[probe] B: manual-vtable RequestEditSession");
    let session2: ITfEditSession = ProbeSession {
        callback: Mutex::new(Some(Box::new(move |ec| {
            println!("[probe] B callback ec={ec}");
            Ok(())
        }))),
    }
    .into();
    let this = Interface::as_raw(&context);
    let vtable = this as *const *const usize;
    let slot = unsafe { *(*vtable).add(3) };
    type RequestEditSessionFn =
        unsafe extern "system" fn(*mut c_void, u32, *mut c_void, u32, *mut i32) -> i32;
    let function: RequestEditSessionFn = unsafe { std::mem::transmute(slot) };
    let mut rr: i32 = 0;
    let hr2 = unsafe {
        function(
            this,
            tid,
            Interface::as_raw(&session2),
            (TF_ES_SYNC | TF_ES_READWRITE).0,
            &mut rr,
        )
    };
    println!("[probe] B hr=0x{:08X} rr=0x{:08X}", hr2 as u32, rr as u32);

    // C: 变体矩阵——在真实编辑会话回调内调用 InsertTextAtSelection，
    // 逐一排除参数嫌疑（NUL 结尾 / dwflags / pprange / ec / 无选区）。
    println!("[probe] C: variant matrix begins");
    let comp_sink: ITfCompositionSink = CompSink.into();
    let insert = context.cast::<ITfInsertAtSelection>()?;
    println!("[probe] ITfInsertAtSelection ok");

    fn run_variant(
        tag: &str,
        tid: u32,
        context: &ITfContext,
        insert: &ITfInsertAtSelection,
        body: impl FnOnce(u32, &ITfInsertAtSelection, &ITfContext) -> Result<()> + 'static,
    ) -> Result<()> {
        let tag_owned = tag.to_string();
        let insert_v = insert.clone();
        let context_v = context.clone();
        let session_v: ITfEditSession = ProbeSession {
            callback: Mutex::new(Some(Box::new(move |ec| {
                println!("[probe] V[{tag_owned}] DoEditSession ec={ec} begin");
                let r = body(ec, &insert_v, &context_v);
                println!("[probe] V[{tag_owned}] body -> {r:?}");
                r
            }))),
        }
        .into();
        let hr =
            unsafe { context.RequestEditSession(tid, &session_v, TF_ES_SYNC | TF_ES_READWRITE) };
        println!("[probe] V[{tag}] session hr={hr:?}");
        Ok(())
    }

    // R0: 读操作 —— 验证 cookie 是否对上下文读有效（先测读，不污染状态）
    run_variant("R0-GetStart", tid, &context, &insert, |ec, _i, c| {
        let start = unsafe { c.GetStart(ec) };
        println!("[probe] R0 GetStart -> {start:?}");
        Ok(())
    })?;

    // R1: GetSelection —— 编辑器类上下文读选择区
    run_variant("R1-GetSel", tid, &context, &insert, |ec, _i, c| {
        let mut sel = [TF_SELECTION::default(); 1];
        let mut count = 0u32;
        let r = unsafe { c.GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut count) };
        println!("[probe] R1 GetSelection -> {r:?} count={count}");
        Ok(())
    })?;

    // R2: 只查询不修改（TF_IAS_QUERYONLY）
    run_variant(
        "R2-QUERYONLY",
        tid,
        &context,
        &insert,
        |ec, insert_v, _c| {
            let wide = to_wide("n");
            let r = unsafe { insert_v.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, &wide[..1]) };
            println!("[probe] V2 result {r:?}");
            Ok(())
        },
    )?;

    // R3: SetSelection 写操作 —— 判断"任何写"都崩，还是仅 InsertTextAtSelection 崩
    run_variant("R3-SetSel", tid, &context, &insert, |ec, _i, c| {
        let start = unsafe { c.GetStart(ec) }?;
        let sel = TF_SELECTION {
            range: core::mem::ManuallyDrop::new(Some(start)),
            style: TF_SELECTIONSTYLE {
                ase: TF_AE_NONE,
                fInterimChar: windows_core::BOOL(0),
            },
        };
        let r = unsafe { c.SetSelection(ec, &[sel]) };
        println!("[probe] R3 SetSelection -> {r:?}");
        Ok(())
    })?;

    // R4: range.SetText 直写 —— 绕过 ITfInsertAtSelection 的写通道
    run_variant("R4-SetText", tid, &context, &insert, |ec, _i, c| {
        let start = unsafe { c.GetStart(ec) }?;
        let wide = to_wide("n");
        let r = unsafe { start.SetText(ec, 0, &wide[..1]) };
        println!("[probe] R4 range.SetText -> {r:?}");
        Ok(())
    })?;

    // R5: 用 GetSelection 返回的 range SetText
    run_variant("R5-SelSetText", tid, &context, &insert, |ec, _i, c| {
        let mut sel = [TF_SELECTION::default(); 1];
        let mut count = 0u32;
        unsafe { c.GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut count) }?;
        let range = core::mem::ManuallyDrop::into_inner(sel[0].range.clone());
        let Some(range) = range else {
            println!("[probe] R5 no selection range");
            return Ok(());
        };
        let wide = to_wide("n");
        let r = unsafe { range.SetText(ec, 0, &wide[..1]) };
        println!("[probe] R5 sel-range.SetText -> {r:?}");
        Ok(())
    })?;

    // V7: QUERYONLY 取 range，再用 range.SetText 写入（完全绕过写入分支）
    run_variant(
        "V7-qry+SetText",
        tid,
        &context,
        &insert,
        |ec, insert_v, _c| {
            let wide = to_wide("n");
            let range =
                unsafe { insert_v.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, &wide[..1]) }?;
            println!("[probe] V7 QUERYONLY range ok");
            let r = unsafe { range.SetText(ec, 0, &wide[..1]) };
            println!("[probe] V7 range.SetText -> {r:?}");
            Ok(())
        },
    )?;

    // V8: 经典 TIP 组合写法 —— 选区 range 上 StartComposition 再 SetText
    let comp_sink_v8 = comp_sink.clone();
    run_variant("V8-compfirst", tid, &context, &insert, move |ec, _i, c| {
        let mut sel = [TF_SELECTION::default(); 1];
        let mut count = 0u32;
        unsafe { c.GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut count) }?;
        let range = core::mem::ManuallyDrop::into_inner(sel[0].range.clone());
        let Some(range) = range else {
            println!("[probe] V8 no selection range");
            return Ok(());
        };
        let cc: ITfContextComposition = c.cast()?;
        println!("[probe] V8 cast ITfContextComposition ok");
        let comp = match unsafe { cc.StartComposition(ec, &range, &comp_sink_v8) } {
            Ok(c) => c,
            Err(err) => {
                println!("[probe] V8 StartComposition err {err:?}");
                return Ok(());
            }
        };
        println!("[probe] V8 StartComposition ok");
        let wide = to_wide("n");
        let r = unsafe { range.SetText(ec, 0, &wide[..1]) };
        println!("[probe] V8 comp range.SetText -> {r:?}");
        // 验证组合范围是否覆盖插入的 "n"（期望 GetText 返回 1 个字符）
        if let Ok(comp_range) = unsafe { comp.GetRange() } {
            let mut buf = [0u16; 8];
            let mut n = 0u32;
            let g = unsafe { comp_range.GetText(ec, 0, &mut buf, &mut n) };
            println!(
                "[probe] V8 comp-range GetText -> {g:?} chars={n} text={:?}",
                String::from_utf16_lossy(&buf[..n as usize])
            );
        }
        let r2 = unsafe { comp.EndComposition(ec) };
        println!("[probe] V8 EndComposition -> {r2:?}");
        Ok(())
    })?;

    // V10: 新 TIP 模式全流程 —— QUERYONLY 取插入点 → StartComposition →
    //       SetText → 校验组合覆盖 → 第二键 comp.GetRange().SetText 更新
    let comp_sink_v10 = comp_sink.clone();
    run_variant(
        "V10-newflow",
        tid,
        &context,
        &insert,
        move |ec, insert_v, c| {
            let wide = to_wide("n");
            let range =
                unsafe { insert_v.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, &wide[..1]) }?;
            println!("[probe] V10 QUERYONLY range ok");
            let cc: ITfContextComposition = c.cast()?;
            let comp = unsafe { cc.StartComposition(ec, &range, &comp_sink_v10) }?;
            println!("[probe] V10 StartComposition ok");
            let r = unsafe { range.SetText(ec, 0, &wide[..1]) };
            println!("[probe] V10 SetText(1) -> {r:?}");
            if let Ok(comp_range) = unsafe { comp.GetRange() } {
                let mut buf = [0u16; 8];
                let mut n = 0u32;
                let g = unsafe { comp_range.GetText(ec, 0, &mut buf, &mut n) };
                println!(
                    "[probe] V10 after-key1 GetText -> {g:?} chars={n} text={:?}",
                    String::from_utf16_lossy(&buf[..n as usize])
                );
            }
            // 第二键：更新组合文本为 "nn"
            let wide2 = to_wide("nn");
            if let Ok(comp_range) = unsafe { comp.GetRange() } {
                let r2 = unsafe { comp_range.SetText(ec, 0, &wide2[..2]) };
                println!("[probe] V10 SetText(2) -> {r2:?}");
                let mut buf2 = [0u16; 8];
                let mut n2 = 0u32;
                let g2 = unsafe { comp_range.GetText(ec, 0, &mut buf2, &mut n2) };
                println!(
                    "[probe] V10 after-key2 GetText -> {g2:?} chars={n2} text={:?}",
                    String::from_utf16_lossy(&buf2[..n2 as usize])
                );
            }
            let r3 = unsafe { comp.EndComposition(ec) };
            println!("[probe] V10 EndComposition -> {r3:?}");
            Ok(())
        },
    )?;

    // ===== 以下为"参考崩溃段"：确认 InsertTextAtSelection 写入分支仍必崩 =====
    // V3: 手写 vtable 写调用，pprange=NULL（NOQUERY 允许 NULL）
    run_variant("V3-ppnull", tid, &context, &insert, |ec, insert_v, _c| {
        let wide = to_wide("n");
        let this_i = Interface::as_raw(insert_v);
        let vtable_i = this_i as *const *const usize;
        let slot_i = unsafe { *(*vtable_i).add(3) };
        type InsertFn = unsafe extern "system" fn(
            *mut c_void,
            u32,
            u32,
            *const u16,
            i32,
            *mut *mut c_void,
        ) -> i32;
        let insert_fn: InsertFn = unsafe { std::mem::transmute(slot_i) };
        let hr_i = unsafe {
            insert_fn(
                this_i,
                ec,
                (TF_IAS_NOQUERY).0,
                wide.as_ptr(),
                wide.len() as i32,
                std::ptr::null_mut(),
            )
        };
        println!("[probe] V3 hr=0x{:08X}", hr_i as u32);
        Ok(())
    })?;

    // V9: 手写 vtable 写调用，NOQUERY + 非空 pprange
    run_variant(
        "V9-ppnonnull",
        tid,
        &context,
        &insert,
        |ec, insert_v, _c| {
            let wide = to_wide("n");
            let this_i = Interface::as_raw(insert_v);
            let vtable_i = this_i as *const *const usize;
            let slot_i = unsafe { *(*vtable_i).add(3) };
            type InsertFn = unsafe extern "system" fn(
                *mut c_void,
                u32,
                u32,
                *const u16,
                i32,
                *mut *mut c_void,
            ) -> i32;
            let insert_fn: InsertFn = unsafe { std::mem::transmute(slot_i) };
            let mut out_range: *mut c_void = std::ptr::null_mut();
            let hr_i = unsafe {
                insert_fn(
                    this_i,
                    ec,
                    (TF_IAS_NOQUERY).0,
                    wide.as_ptr(),
                    wide.len() as i32,
                    &mut out_range,
                )
            };
            println!(
                "[probe] V9 hr=0x{:08X} out_range={:p}",
                hr_i as u32, out_range
            );
            Ok(())
        },
    )?;

    // V1: cch=1（去掉结尾 NUL）——API 约定 cch 不含终止符
    run_variant("V1-cch1", tid, &context, &insert, |ec, insert_v, _c| {
        let wide = to_wide("n");
        let r = unsafe { insert_v.InsertTextAtSelection(ec, TF_IAS_NOQUERY, &wide[..1]) };
        println!("[probe] V1 result {r:?}");
        Ok(())
    })?;

    // V2: cch=1 + dwflags=0（要回 pprange）
    run_variant("V2-query", tid, &context, &insert, |ec, insert_v, _c| {
        let wide = to_wide("n");
        let r = unsafe {
            insert_v.InsertTextAtSelection(ec, INSERT_TEXT_AT_SELECTION_FLAGS(0), &wide[..1])
        };
        println!("[probe] V2 result {r:?}");
        Ok(())
    })?;

    // V4: 先 SetSelection 到文档起点，再 cch=1 插入
    run_variant("V4-sel", tid, &context, &insert, |ec, insert_v, c| {
        let start = unsafe { c.GetStart(ec) }?;
        let sel = TF_SELECTION {
            range: core::mem::ManuallyDrop::new(Some(start)),
            style: TF_SELECTIONSTYLE {
                ase: TF_AE_NONE,
                fInterimChar: windows_core::BOOL(0),
            },
        };
        match unsafe { c.SetSelection(ec, &[sel]) } {
            Ok(()) => println!("[probe] V4 SetSelection ok"),
            Err(err) => {
                println!("[probe] V4 SetSelection err {err:?}");
                return Ok(());
            }
        }
        let wide = to_wide("n");
        let r = unsafe { insert_v.InsertTextAtSelection(ec, TF_IAS_NOQUERY, &wide[..1]) };
        println!("[probe] V4 result {r:?}");
        Ok(())
    })?;

    // V5: 使用 CreateContext 返回的 ec(=0) + cch=1
    run_variant("V5-ec0", tid, &context, &insert, |ec, insert_v, _c| {
        let _ = ec;
        let wide = to_wide("n");
        let r = unsafe { insert_v.InsertTextAtSelection(0, TF_IAS_NOQUERY, &wide[..1]) };
        println!("[probe] V5 result {r:?}");
        Ok(())
    })?;

    unsafe { tm.Deactivate()? };
    println!("[probe] done");
    Ok(())
}
