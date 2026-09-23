# Agent Note: TSF 组合写入路径避开 InsertAtSelection 写入分支

Status: implemented

[English](2026-09-23-tsf-insert-at-selection-write-path-crash.md) | 中文

## Problem

在验收虚拟机（Windows Server 2019，msctf.dll 17763）上，输入法在每次记事本会话中
输入第二个键即崩溃，访问违例 `0xC0000005`（`MSCTF+0x64314`）。本机 Windows 11
（msctf.dll 26100）在最小 TSF 宿主探针中同样崩溃。`examples/tsf_min_host.rs` 的
变体矩阵把故障定位到 `ITfInsertAtSelection::InsertTextAtSelection` 的**写入分支**
（`dwFlags` 不含 `TF_IAS_QUERYONLY`）配合**非空 `pprange`**——即文档建议的
“返回插入文本落点”用法。两个 msctf 版本在同一调用上崩溃，尚未触达项目代码的
range 指针。而 `TF_IAS_QUERYONLY` + 空 `pprange` 的只读定位、`ITfRange::SetText`、
`ITfContextComposition::StartComposition`、`ITfRange::GetText` 在所有变体中均正常
返回。

## Decision

TIP 不再使用 `InsertTextAtSelection` 的写入分支：

- 开始组合：`InsertTextAtSelection(ec, TF_IAS_QUERYONLY, text)` 且 `pprange` 为
  **空**，仅用来定位插入点 range；由 `ITfRange::SetText` 写入拼音；
  `ITfContextComposition::StartComposition` 在该 range 上开启组合；失败时结束并
  清理组合。
- 更新已有组合：复用存储的 `ITfComposition::GetRange()` + `ITfRange::SetText`
  （与崩溃前设计一致）。
- 带文本提交：同样用 QUERYONLY 插入点 range + `SetText`，然后结束组合。

每一步都有 `comp-insert-begin/ok`、`comp-ccomp-begin/cast-ok`、
`comp-start-ok/err`、`comp-settext-ok/err` 日志，便于远程 VM 通过
`C:\zhu-ye-test\tsf-debug.log` 验收该路径。

## Alternatives considered

**保留写入分支并传空 `pprange`。** 否决：变体矩阵显示写入标志 + 空 `pprange`
返回 `E_INVALIDARG`（0x80070057）且不写入任何内容——写入分支在两个被测 msctf
版本上都不可用。

**改用 `ITfInsertAtSelection::InsertEmbeddedAtSelection`。** 否决：它面向嵌入
对象而非纯文本，且不解决写入分支本身的故障。

**把崩溃上报为 msctf 缺陷并等待。** 否决：输入法必须现在交付；QUERYONLY +
`SetText` 是受支持且有文档的组合，消除了对损坏分支的依赖。

## Consequences

组合、提交与候选管线在两个 msctf 版本上都不再崩溃。VM 验收输入 `nihao` + 空格、
多次组合会话全程无崩溃；组合 range 正确覆盖新写文本（`GetText` 字符数 `1` → `2`
已验证）。该写入分支陷阱记为后续任何文本插入代码路径的注意事项：定位插入点用
`TF_IAS_QUERYONLY`，写文本走 `ITfRange::SetText`。

T-011 保持进行中，直至 T-009–T-013 的完整 VM 端到端验收关闭；状态见任务清单。
