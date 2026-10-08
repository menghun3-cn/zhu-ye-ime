# Agent Note: 托盘状态图标方形化——品牌橙底白字"中/英"（T-123）

Status: implemented

[English](2026-10-08-tray-icon-square.md) | 中文 | 中英镜像

## Problem

批六交付的托盘状态图标（`crates/zhu-ye-tray/assets/tray-zh.ico` /
`tray-en.ico`）是**白色"中"/"英"字形 + 全透明底**（32×32 单帧，四角与
边中点 alpha=0）。Windows 11 对透明托盘图标套用圆形遮罩后，视觉上呈现
"圆形图案、中间中/英字"。用户目视反馈（批八）：**托盘图标改为方形图案**。

## Decision

给两枚托盘图标加**品牌橙方形底**，与 DLL 内嵌指示器图标同款视觉：

- 底色 `#E5881E`（品牌橙，同 `zhu-16.png` / `zhu.ico` 设计源）；
- 直角方形满底（不透明，四角 alpha=255），居中白"中"/白"英"粗体字
  （微软雅黑 Bold）；字形不再直接悬空于透明底；
- 帧结构 16/32/48 三帧 PNG ICO（Vista+ `LoadImageW` 原生支持 PNG 帧；
  托盘按 `SM_CXSMICON` 取 16px，高 DPI 放大取 32/48）；
- 生成脚本 `scripts/make-tray-icons.ps1` 入库，图标可复现（System.Drawing
  绘制 + ICO 容器手工打包，无第三方工具）；
- `build.rs` 的 101/102 资源语义不变；托盘机制（常驻进程、状态文件桥、
  单实例互斥）全部不动。

## Alternatives considered

- **保持透明底**。否决：正是白色字形 + 透明底在 Win11 圆角遮罩下呈现
  "圆形居中字"的观感来源，与用户诉求相反。
- **圆角方形底**。否决：用户字面要求"方形"，且直角与 `zhu.ico` 家族
  图标一致（`zhu-16.png` 四角不透明）。
- **改用深灰/黑色底区分状态**。否决：保持品牌统一，状态仍靠"中/英"
  字形区分（与系统指示器"竹/中"同构），不引入第二套托盘配色。

## Consequences

- 托盘图标在任意尺寸（16/32/48）下都是一块橙色方形 + 白色中/英字；
  系统圆角遮罩只影响图标外轮廓最外角，主体方块观感不再为"圆形"。
- `zhu-ye-tray.exe` 体积小幅增大（内嵌三帧 PNG，约 +2KB）。
- 图标为生成产物，来源与命令记录于脚本头部；改动资产后
  `cargo build -p zhu-ye-tray --release` 即重新嵌入。

### Verification

- 像素验证（GDI+ 加载新 ICO）：四角与边中点 = `(229,136,30)` alpha=255
  （方形满底），中心白色（字形居中）；两个图标 whitePx 190/261。
- 构建后从 `target\release\zhu-ye-tray.exe` 提取出全部 6 个 PNG 帧
  （3×zh + 3×en），与生成资产逐一字节比对一致。
- 门禁：`cargo fmt --check`、`cargo clippy -p zhu-ye-tray -- -D warnings`、
  `git diff --check`、生成脚本 ParseInput 解析全部通过。
- 部署：用户机 worker `copy-exe` 覆盖 `bin\zhu-ye-tray.exe`
  （262,144B，sha256 `BC6CFFC9C34F9710608C08E879AA78A3539F458F99FD8B1CE3560A236C217580`）、
  托盘重启后中/英状态轮询正常；VM 同步替换同一二进制并截图核对：
  **托盘槽像素探针**——16×16 橙色方块 bbox (x62..77, y40..55) + 白色字形
  （中态橙 190px/白 168px，英态橙 174px/白 165px），中/英两态均为方形橙底
  白字（截图：`target/t122-deploy/t123-shots/tray-zh.png` / `tray-en.png`）。

Related: [批六系统托盘"中/英"状态图标](../../implemented/feature/2026-10-07-system-tray-mode-icon.md)
——机制未变，仅图标视觉由"白字透明底"更新为"橙底白字方形"（该笔记同一
批量维护）。
