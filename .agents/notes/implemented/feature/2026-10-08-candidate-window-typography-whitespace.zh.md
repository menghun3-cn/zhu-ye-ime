# Agent Note：候选窗排版与留白美化（T-126）

Status: implemented

## 问题

用户对候选窗的反馈（T-126）：拼音行（`nǐ hǎo`）与汉字（`你好`）几乎贴在一起没有呼吸；声调符号（ǐ/ǎ）渲染偏移、割裂（字体问题）；拼音行字号偏大、颜色偏深，不像辅助提示；序号/拼音/汉字对齐不整齐；顶部输入缓冲区（`nihao` + 灰色 `ni hao`）显得局促。用户给出的要求（CSS 风格参数，映射到 GDI 自绘窗口）：

1. 拼音↔汉字垂直间距 ≈4-8px + 候选项上下内边距；
2. 拼音行指定更优字体栈（用户建议 `"Segoe UI", "Helvetica Neue", "Pinyin", Arial, sans-serif`），确保声调贴元音；
3. 拼音调小（12/13px）变淡（≈#888）作辅助；汉字 16-18px 加粗、红/蓝高对比作视觉焦点；
4. 序号/拼音/汉字水平对齐、拼音宽度与汉字对齐（或整体居中），拼音不乱跑；
5. 顶部输入缓冲区垂直居中与内边距更舒展。

## 决策

### 布局（candidate_ui.rs `CandidateMetrics`）

- `pin_line_gap` 首版 4→8dp（用户 4-8px 档取上限），用户目视后要求
  "拼音和中文距离减少一半"，定稿 **→4dp**（见下方"用户目视反馈"段）。
- `row_height` 36→48dp、`row_gap` 2→4dp：主文本区（定稿 4dp gap 下
  48−17=31dp）容纳 16dp 字形、下方留空 —— 候选行有呼吸。
- `padding_y` 10→12dp（面板上下留白）、`header_height` 38→48dp（顶部输入
  缓冲区：16dp 输入串垂直居中、上下各 ≈16dp 舒展留白）。
- 九行面板 398→536dp（间距收窄不影响：行高是行距来源）。

### 字体（candidate_window.rs）

- 主文本：宋体 16dp `FW_NORMAL`→`FW_SEMIBOLD`（16px 加粗 —— 汉字成为视觉
  焦点；未选 #1E88E5 蓝/选中 #D32F2F 红保持）。宋体无真粗体，GDI 合成加粗
  在 16px 下渲染干净，但**墨迹横向外扩** —— 见下方截断回归修复。
- 拼音：宋体 13dp 半粗→**Segoe UI 13dp 常规**。`create_font` 新增 `face`
  参数。Segoe UI 对 precomposed 声调字符（ǐ U+01D0、ǎ U+01CE 等）有形态完整
  的单字形，声调贴合元音，修掉宋体的割裂偏移。

### 颜色（candidate_ui.rs `theme`）

- 浅色 `pin` #555555→**#888888**（用户指定辅助灰；白底对比度 ≈3.4:1，刻意
  弱于译文/序号 #999999）。深色 #C9C9C9 与高对比 gray_text 不变。这是对
  T-122"加深"决策的**刻意反转**—— 用户现在明确要拼音作浅色辅助提示而非
  高对比细节；可读性由 Segoe UI 字形保障。

### 对齐（candidate_ui.rs 纯函数 + paint）

- `CandidateMetrics::pin_row_rect(row, main_col, pin)`：拼音矩形 `left =
  main_col.left`（序号列之后拼音与汉字共享同一左缘 —— 严格上下对齐），
  top=行顶、bottom=行顶+拼音字高（**纯字形区，间距不含** —— 见用户目视
  反馈段），`right = max(main_col.right, main_col.left +
  estimate_text_width(pin, 13dp))` 收敛到行右缘。短拼音与汉字列同宽；长拼音
  （如 `zhang hao`）完整容纳不截断。
- `paint` 将拼音画入该矩形，主文本/译文区从 `pin_rect.bottom + pin_line_gap`
  起、高=`font_height`。序号仍右对齐标记列（T-122），有拼音行时随主文本带
  下移 —— 数字与汉字同竖直中心。

## 备选方案

- **拼音继续用宋体/组合变音符**：宋体对带调字符的字形就是割裂源头，组合
  序列更糟。Segoe UI（所有受支持 Windows 都有）是零风险修复，产品不发字体
  文件。
- **主字重 FW_BOLD**：16px 下过重；SEMIBOLD 已明显加粗且 ClearType 顺滑。
- **拼音 12dp**：用户允许 12/13；13 保证声调可辨。
- **拼音右缘无条件=汉字列宽**：`zhang hao` 式纠错拼音比汉字宽时会被截断；
  max(列宽, 内容宽) 兼顾常见情况对齐与长情况完整。
- **间距留在拼音带内（旧式带底=顶+字高+间距）**：`draw_text` 把文本在矩形内
  垂直居中，带高高于字形时把间距当居中余量吸收 —— 框距 8 与 4 只显示同样的
  ≈9.5px 墨迹间距。改纯字形区（底=顶+字高，主文本顶=底+间距）后**视觉间距**
  才真正由框距决定。

## 后果

- 候选行可见地"松了"（48dp 行、拼音视觉 4-8px 档、4dp 行距）；顶部缓冲区
  舒展（48dp 居中）；拼音行成为浅色辅助层级（Segoe UI 13px #888）；汉字承担
  视觉重量（宋体 16px 半粗、蓝/红）；序号与汉字同竖直中心。
- 面板 398→536dp 高（9 行）；宽度逻辑不变（T-124 内容自适应宽未动）。

## 验证

- 单元测试（candidate_ui.rs）：`pin_line_gap` 精确 4（4-8px 档内）；拼音
  字形区+主文本区适配 48dp 行；`pin_row_rect` 左缘=汉字列左缘、短拼音跨宽恰
  为汉字列宽、`nǐ hǎo`（宽于 你好）按估算宽完整容纳；主文本区顶=字形区底
  +gap；浅色 `pin == #888888`。candidate_ui 24/24、zhu-ye-ui 35/35；
  workspace 全绿（core 326、ime 215、settings 103 等）；fmt/clippy -D
  warnings/diff-check 通过。
- `candidate-demo --shot`（96dpi 浅色，定稿参数）像素探针：行 pitch=52；
  页眉 `nihao` 蓝 span 53px 完整（带 y31..41，中心 36/48）；row1 蓝三字
  span 50px 完整、row0 红两字 span 33px；序号红 y 中心 85 vs 汉字 84
  （同带）；拼音（Segoe UI）墨迹 y117..123 → 汉字墨迹 y129..144 =
  **视觉 gap 6px**；拼音灰落 110-158 档（无 #555 深灰）；蓝 #1E88E5/红
  #D32F2F 精确命中。
- 125%（用户机缩放）截图：窗口 450×695px，渲染一致。
- PR #136 合入 develop（2845800）；release `zhu_ye_ime.dll`（2,198,016 B，
  sha8 FEAEF72E，TSF 身份 7/7）经 `copy-dll-ver` 部署用户机
  （`zhu_ye_ime_FEAEF72E.dll`，CLSID/IconFile 指针切换，ctfmon 重启）。

## 用户目视反馈（首版部署当天）

用户真机输入 `nihao` 看到候选词被截成 `你...`。根因：T-126 主文本改
`FW_SEMIBOLD`，宋体无真粗体、GDI 假粗使墨迹横向外扩（~1px/字），而
`estimate_text_width` 按常规 advance 宽给矩形 → `DT_END_ELLIPSIS` 把最右字
换成省略号。修复：`estimate_text_width` 改用保守系数（ASCII 0.55→0.58em、
CJK 1.0→1.06em，`fit_text` 同步）—— 所有消费方矩形（行分栏、页眉、拼音）
都放得下真实墨迹。页眉输入串 `nihao` 有同样的潜在问题，现验证完整
（53px 无省略号）。

用户另提两点：

1. **"序号和中文那行对齐，不要和拼音对齐"** —— 序号随主文本带（show_pin
   时 `marker_rect.top/bottom`=汉字带）与汉字同竖直中心；无拼音行保持整行
   居中。
2. **"拼音和中文距离减少一半"** —— `pin_line_gap` 8→4，且 `pin_row_rect`
   改为**纯字形区**（底=顶+拼音字高，间距不含）、主文本区顶=字形区底+
   `pin_line_gap`。如备选方案所述，这正是让改动"看得见"的关键：实测墨迹
   gap 6px（Segoe UI 字形顶部比 13px 格高缩 ~2px）对修前 9.5px —— 约减半、
   落用户原 4-8px 档内。与截断修复同一分支/PR 落地（fix/T-126 后续）。

Related：[候选窗水平布局与拼音行排版](../../implemented/feature/2026-10-08-candidate-window-layout-pinyin-typography.zh.md)（T-122）保持 active —— 本 note 原位更新其拼音参数（字重/颜色/间距/字体、主文本加粗、行高）；序号右对齐与独立 `pin` 主题键决策不变。

## 部署通道备忘（流程、电池供电）

用户机部署通道是计划任务 `ZhuYeImeElevated`（`InteractiveToken` +
`RunLevel HighestAvailable` + `DisallowStartIfOnBatteries=true`）。笔记本
**电池供电**时该任务永远 `Queued`—— 两次验证（21:53 与 21:58）Last Result 0、
日志不动。采用的工作区：`Start-Process powershell -Verb RunAs -File
<tsf>\elevated-worker.ps1`（一次 UAC 确认）—— worker 的单次请求/响应模型
直接跑提权进程即可，无需计划任务。想靠 `schtasks /Create /XML` 改掉电池
限制在 Medium IL 下被拒，所以下次部署记住：UAC 直跑或插电源二选一。
