# Agent Note: 候选窗水平布局与拼音排版

Status: implemented

## Problem

用户在已交付候选窗的目视检查中（T-122，批七）报告了两个可读性问题。

1. 候选序号与候选词之间有额外留白。词列起点固定为 `padding_x + marker_width`，而序号在该宽列内**左对齐**，单位数序号（1-9）后面有约 19px 空档才到词。
2. 候选词上方的拼音/声调行（`nǐ hǎo`）渲染过淡：11px 宋体常规字重，使用译文/序号那档 `secondary` 浅灰（浅色下 #999999），与下方汉字仅 3px 间距。小尺寸下显得模糊拥挤。

用户要求序号与候选词间距落在 4-8px（词向左移），拼音行提高对比度、略增字号、加重字重、加大与下方汉字的间距。

## Decision

### 序号与词的间距：序号右对齐 + 固定间距

`CandidateMetrics` 保留 `marker_width` 作为词左偏移，但由 26dp 收窄到 22dp，新增 `marker_text_gap`（6dp，落在用户 4-8px 档内）；绘制循环改用 `draw_text_right` 把序号右对齐进标记列，且其右缘先让出 `marker_text_gap`。因此间距对任何位数（"1" 到 "18"）恒定，不再随数字宽度漂移。96dpi 下词起点由 x=38 左移到 x=34（12 内边距 + 22）。

### 拼音行排版与专属颜色

`CandidateMetrics::pin_font_height` 由 11dp 增至 13dp、`pin_line_gap` 由 3dp 增至 4dp；`create_font` 增加 `weight` 参数，拼音字体以 `FW_SEMIBOLD`（600）创建，主文本保持 `FW_NORMAL`。拼音行不再复用译文/序号的 `secondary` 色：`CandidateUiTheme` 新增第 8 键 `pin`——浅色 #555555（深灰，白底对比度 ≈7:1，仍弱于蓝色主文本）、深色 #C9C9C9（更亮灰）、高对比度沿用系统 `gray_text`。主题文件配色节（`CandidatePalette`/`parse_candidate`）新增可选第 8 键 `pin`；缺键经既有 `theme_with_candidate` 叠加回退默认，已有主题文件完全不受影响。

两个布局常量均有新单测覆盖（间距在 4..=8、词起点早于旧 26dp、拼音字号 >12dp 且仍低于主字 16dp、`pin_band + font_height ≤ row_height`）。

## Alternatives considered

- **只缩窄标记列、序号保持左对齐。** 单位数能得到理想间距，但翻页后两位数字（"10".."18"）会溢出列宽撞到词。右对齐 + 固定间距对所有页都正确。
- **加深共用的 `secondary` 色。** 这会把译文、提示、页脚一并加深，改变整个"弱化文本"层级，违背 T-032 对这些元素的配色约定。独立 `pin` 键（含主题文件支持）只改变拼音行一处的对比度。
- **拼音字体用 `FW_BOLD`。** 宋体没有真粗体，GDI 合成的伪粗在 13px 下发糊。`FW_SEMIBOLD`（600）清晰而不糊。
- **抬高 `row_height` 给拼音行更多空间。** 拼音带（13+4=17dp）加 16dp 主字形在 36dp 行内仍放得下；加高整行只增面板高度，无可读性收益。

## Consequences

- 序号与词的间距在所有页码下稳定在 ~6px（96dpi），满足用户 4-8px 要求；词起点左移 4px。
- 拼音/声调小字清晰可辨：更深灰（深色下更亮灰）、13px、半粗、词上方留 4px 行距。
- 主题作者可在主题文件 `candidate` 节可选添加 `"pin"`；旧文件不受影响（缺键 → 默认）。
- `CandidateUiTheme` 变为 8 键结构，所有构造点已同步（`theme()` ×3、`theme_from_system_colors`、`theme_with_candidate`）；candidate-demo 演示项携带带调拼音（你好 → nǐ hǎo），截图可覆盖拼音行。

### Verification

- `candidate-demo --shot` BMP 像素取证（96dpi 浅色）：首候选词蓝色左缘 x=38 → x=34；深灰（0x55 档）像素 0 → 377（拼音行改色生效），0x99 浅灰档仍驱动译文/序号。
- 单测：`candidate_ui` 20/20（含新增间距/排版断言）；`theme_file` 解析测试覆盖 `pin` 键与缺省（`None`）回退。
- 门禁：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test --workspace`、`git diff --check` 全绿；host-e2e 复跑全绿。
- 以 PR #128 合入 develop（merge 3786e00）；发布重建 run 37735245588 成功，v0.1.2 zip/资产刷新（zip 22,653,533 B）。
- 本地构建 release DLL（AEE25CA1，2,196,480 B，TSF 导出校验通过）经用户机提权 worker `copy-dll-ver` 部署（CLSID InprocServer32 + 语言档 IconFile 指针切换，落地 `C:\Program Files\zhu-ye-ime\tsf\zhu_ye_ime_AEE25CA1.dll`，ctfmon 重启）；`verify-tsf-identity` 7/7 PASS。
- VM（Server 2019）经发行包契约安装（install.ps1，f143c14e → `tsf\zhu-ye-ime-aee25ca1.dll` 升级），并在 VM 截图上复跑同一像素探针：词左缘 x=34、序号-词视觉间距 8px（4-8px 档内）、深灰拼音像素存在（before 0 → after 两行 377px）。
- 说明：发布 zip 资产因网络侧 SNI 限制无法从 `release-assets.githubusercontent.com` 下载（可达 GitHub 边缘 IP 均拒绝该域）；本地构建 DLL 与 CI 产物同 commit 同工具链二进制等价，且本批未改词典三件套（SHA 不变），故 VM 包复用既有 v0.1.2 便携 zip 仅替换 `bin\zhu_ye_ime.dll`。

Related: [批四候选拼音截断与声调表](../../implemented/feature/2026-10-07-acceptance-fix-batch-4.md) 保持活跃——本笔记只改拼音行的外观参数，不涉及按行截断或声调来源语义。
