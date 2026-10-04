//! 本地探针：直接获取真实 `ITfThreadMgr`，验证
//! `QueryInterface(IID_ITfKeystrokeMgr)` 途径可取到按键管理器
//! （并保留手写槽 14 的回归对比，默认禁用）。
//! 仅供诊断使用，不属于交付物。

use std::ffi::c_void;

use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::TextServices::{CLSID_TF_ThreadMgr, ITfThreadMgr};
use windows_core::Interface;

fn main() {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .expect("CoInitializeEx 失败");
    }
    let tm: ITfThreadMgr =
        unsafe { CoCreateInstance(&CLSID_TF_ThreadMgr, None, CLSCTX_INPROC_SERVER) }
            .expect("创建 ITfThreadMgr 失败");

    let this = Interface::as_raw(&tm);
    let vtable = this as *const *const usize;
    println!("thread_mgr this={this:p}");

    // 打印 0..=16 槽，便于核对接口表顺序
    for i in 0..=16usize {
        let v = unsafe { *(*vtable).add(i) };
        println!("slot {i}: {v:#x}");
    }

    // 方式 A：QueryInterface(IID_ITfKeystrokeMgr) —— TSF 关键规范途径
    match tm.cast::<windows::Win32::UI::TextServices::ITfKeystrokeMgr>() {
        Ok(km) => println!("QI ITfKeystrokeMgr: OK -> {km:?}"),
        Err(e) => println!(
            "QI ITfKeystrokeMgr: ERR {e:?} (0x{:08X})",
            e.code().0 as u32
        ),
    }

    // 方式 B：手写槽 14（原有假设）——预期崩，注释掉避免退出
    // （保留代码供回归对比，但默认不执行）
    if std::env::var("PROBE_SLOT14").is_ok() {
        const SLOT: usize = 14;
        type GetKeystrokeMgrFn = unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> i32;
        let slot_fn: usize = unsafe { *(*vtable).add(SLOT) };
        let function: GetKeystrokeMgrFn = unsafe { std::mem::transmute(slot_fn) };
        let mut out: *mut c_void = std::ptr::null_mut();
        let hr = unsafe { function(this, &mut out) };
        println!(
            "slot{SLOT} GetKeystrokeMgr hr=0x{:08X} out={:p}",
            hr as u32, out
        );
    }
}
