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

## 拼音字形带改用真实 tmHeight（T-130，用户反馈"拼音下方被截断"）

用户 2026-10-09 反馈：部分词汇上方拼音底部被截断。根因：`pin_row_rect`
的拼音带按 `pin_font_height`（13px）取高，而这是**字符高**（`lfHeight = -13`）；
Segoe UI 13px 的真实 `tmHeight` ≈ 16px。`DrawTextW` 按矩形裁剪，`g`/`j`/
`p`/`q`/`y` 等字母的下伸部（descender，约 3px）被削平，而行内定位仍按
13px 带计算。

修复（全部留在 `candidate_window.rs`，布局常量不动）：pin 字体创建后
`query_font_tm_height` 用临时兼容 DC 读 `GetTextMetricsW` 的 `tmHeight`，
窗口状态存 `pin_tm_height`；paint 里拼音**绘制矩形**与由此推导的主文本/
序号顶边统一按 `pin_tm_height - pin_font_height` 扩展——字形带=真实
tmHeight，墨迹与下方汉字视觉间距仍为 `pin_line_gap`（4px）。

像素证据（96dpi 浅色，`gāo fēng` 演示行）：修前拼音墨迹 y=234..248
（在 13px 线处被切平）；修后 y=237..252（`g` 下伸完整），与下方主文本
间距不变。

## 候选行内容带垂直居中（T-132，用户反馈"选项行内部下留白过多"）

用户 2026-10-09（T-131 部署后随即反馈）：每个候选词汇（选项行）内部的
下留白过多，显得松散。

根因：带拼音的行把内容带锚在行**顶**——拼音字形带（13px + ~3px tmHeight
展开）→ `pin_line_gap`（4px）→ 16px 主文本带，共 36px，行高 48 → 底部
悬空约 12px（再加 `row_gap` 4px），观感"松散"。无拼音行本就整行垂直
居中（`DT_VCENTER`）。

修复（T-132，只动 `candidate_window.rs` paint；面板尺寸/分页不变）：新增
纯函数 `CandidateMetrics::pin_band_top(row, pin_tm_height)` 返回内容带顶，
使整条内容带（拼音带 + 间距 + 主文本带）在行内垂直居中；拼音绘制矩形、
主文本/译文矩形、序号矩形（按 T-126"序号跟中文对齐"随主文本带）都从该顶
推导。96dpi 常量下内容带由 0px/12px 变为上下各约 6px。

像素证据（96dpi 浅色，带拼音演示行 `nǐ men hǎo`）：修前墨迹 y=116..148
（行 112..160，上留白 ≈4px、下留白 ≈11px）；修后 y=121..153（上 ≈9px
含拼音字形上留、下 ≈6px）——内容带整体下移约 5px 且行内均衡。无拼音行
（译文模式/拼音开关关闭）不受影响（16px/16px 原本居中）。

*（T-138 后来把带拼音行的这种居中改为顶部对齐 1px——见下文「音标上方
内边距收紧到 1px (T-138)」节。）*

## 页眉高度压缩 30%（T-136，用户反馈"候选框顶部留白过多，视觉重心偏下"）

用户 2026-10-09：候选框顶部留白过多、视觉重心偏下；要求把顶部拼音区
高度压缩 30%，并把第一项候选词紧贴拼音区（减少两者垂直间距）。

修复（T-136）：`CandidateMetrics::new` `header_height` 48→**34**dp
（48×0.7=33.6→34；16px 页眉字高 + 上下各 ≈9px 空间）。`row_rect` 本就是
`top = padding_y + header_height`（页眉底与首行顶共边、无显式 margin），
压缩页眉即整体上移——首行行顶 60→46、九行面板 536→522（−14dp）。行高
与行内内容带居中（T-132）及 `padding_y` 保持不动（用户未点名）。验证：
candidate_ui 测试全绿（面板尺寸断言 536→522 随几何调整）、workspace 全绿；
96dpi demo 像素探针——页眉字形墨迹 y29..34（34 区内垂直居中）、首行候选
墨迹顶 72→58（上移 14px）、面板高 542（522+页脚 20）。

## 序号列贴左边缘、间距收紧到 2px（T-137，用户"序号离左边缘还是太远"）

用户 2026-10-09（T-136 部署后随即反馈）：序号离左边缘还是太远，希望缩小、
距离 2px 即可；序号容器只需能容下 2 位数的宽度。此指令覆盖 T-122 定的
4-8px 序号-词间距档。

修复（T-137）：`CandidateMetrics` 新增 `marker_left` = 2dp——序号列起点
改为贴面板左缘 x=2（高亮块内缩 1px + 边框 1px），不再从 `padding_x`(12)
缩进。`marker_width` 22→**20**dp（两位数字 ≈18px 右对齐 + 余量，贴合
"容器能容下 2 位数即可"）、`marker_text_gap` 6→**2**dp（"距离2px"）。
`marker_rect` 与 `row_split` 都以 `marker_left + marker_width` 推导词起点，
词起点 34→22；序号保持右对齐（个位对齐）+ 恒定 2px 间距——与 T-122
右对齐理由一致，仅仅整体贴左。96dpi 几何：两位序号墨迹从 x≈3 起（贴边）、
一位"1"从 x≈13 起（右对齐固有代价：改左对齐会使个位错位或间距爆大）、
序号-词墨迹间隙 ≈3-4px、首词墨迹 x≈23。验证：candidate_ui 25/25（贴边/
2px/容两位断言）+ workspace 全绿；demo 96dpi 像素探针（词墨迹 34→23）。

T-122 右对齐理由保持 active；其列参数原位更新（词起点 x=34→22、间距 6→2dp）。

## 音标上方内边距收紧到 1px（T-138，用户"音标上方的内边距再减少到只剩下1px"）

用户 2026-10-09（T-137 部署后随即反馈）：候选行内音标（带调拼音行）上方的
呼吸空间应缩小到只剩 1px。替代 T-132 对带拼音行的整带垂直居中。

修复（T-138）：`CandidateMetrics::pin_band_top` 返回 `row.top + 1`（原居中
`row.top + (行高−内容带)/2`，96dpi 为 +6）；内容带高于行高仍贴行顶。
paint 的主文本/译文/序号矩形都从 `band_top` 派生（与 T-132 同源），整带
自动跟随。96dpi 几何：拼音墨迹 58→52（上移 6px），留白由 6/6 变为
**上 1/下 11**（内容带 = 拼音 16 + 间距 4 + 主字 16 = 36，行高 48）。
无拼音行（译文模式/拼音开关关闭）仍 `DT_VCENTER` 整行居中不受影响。
主文本下方 11px 是顶部对齐的取舍——若用户目视嫌松，下一杠杆是收行高或
在偏上带内重居中。验证：candidate_ui 25/25（测试改写为 +1/下 11 断言 +
tiny 行保护）、workspace 全绿；demo 96dpi 探针——行 0（顶 46）：拼音墨迹
y52..60（band 顶 47，墨迹距行顶 ≈5-6px 含 Segoe UI 13px 字形顶部空隙）、
主文本墨迹 y68..83（下方 11px）。

## 候选卡片外边距归零、页眉贴顶 1px（T-140，用户"候选词卡片内已经有内边距，外边距直接改为0；候选框输入的最上边留白太多，改为距离最上面1px"）

用户（2026-10-09，T-139 部署后）：两处空白压缩——(1) 候选**卡片与卡片**的
外边距改为 0（每行候选词自身已带内边距）；(2) 输入区（页眉组合串带）的
**最上边留白**压缩到距离窗口顶部 1px。两者都是 `CandidateMetrics` 的纯
布局值改动，行内绘制算法不变。

改动（`candidate_ui.rs`）：

- 新增 `header_top: dp(1.0)`——页眉区从窗口顶 1dp 处开始（原为
  `padding_y`=12）；`header_rect` 与 `row_rect` 的行顶推导由
  `padding_y + header_height` 改为 `header_top + header_height`。
- `header_height` 34→25dp（1 顶距 + 16 字带 + ≈8 下缓冲）。组合串不再在
  页眉区内垂直居中：`draw_header_mixed` 基线改为 `rect.top +
  main_tm.tmAscent`（字带顶 = rect.top）；无分隔符路径从 `draw_text`
  （DT_VCENTER）改用新增的 `draw_text_top`（字带顶对齐，DrawTextW 不带
  DT_VCENTER 即 DT_TOP；emoji 彩色路径与 draw_text 共用）。页眉右侧提示
  同样顶对齐。
- `row_gap` 4→0（行与行紧贴；行自身内边距——拼音带上方 1px、主文本下方
  11px，T-138——仍把墨迹隔开）。
- `panel_size` 顶部项由 `padding_y * 2` 改为
  `header_top + header_height + padding_y`（底部内边距保持 12）。

96dpi 几何：面板高 542→490（页眉 −20、行距 −32）；首行顶 46→26（上移
20）。像素探针（t140-xian-light）：页眉墨迹自 y=1 起（y0 为 1px 边框线；
字带顶对齐后墨迹 y4..14）；T-139 直撇上移至 y4..6（仍 x31..32）；行1
拼音/主文本墨迹 31..40 / 48..63；行2 顶 = 行1 底（74）——行间距 0 确认。
深色主题复渲。页眉区的垂直居中（T-136）被本项取代（保留原节——同属
留白叙事线）；行内容带的顶对齐（T-138）不变。

与 T-139 同轮 PR 跟进；待用户真机目视。

*T-144 修订（2026-10-09，用户"候选框最上面的拼音距离上面从1px改为2px"）：*
`header_top` 1→2dp。页眉区及其派生矩形（首行顶、`panel_size`）整体下移
恰好 1dp：96dpi 面板高 490→491。像素探针（t144-candidate.bmp，同一
`--sep` 输入）：页眉字形墨迹 y4..14 → **y5..15**（整体 +1；边框线仍在
y0，字带顶 y2 + 字形顶部空隙 ≈3px）——拼音组合串现在距窗口顶 2px。
新增单测「页眉贴顶距离为2px且整条候选带随之下移」锁定 `header_top == 2`、
`header_rect().top == 2`、首行顶 = 27 与面板高 39（0 行）/ 491（9 行）。

*T-146 修订（2026-10-09，用户"候选框用户输入拼音距离顶部边缘改为3px"）：*
`header_top` 2→3dp，取代 T-144 数值。所有派生矩形整体下移恰好 1dp：
96dpi 面板高 491→492；拼音组合串现在距窗口顶 3px。T-144 单测就地改名
并更新为「页眉贴顶距离为3px且整条候选带随之下移」，锁定 `header_top == 3`、
`header_rect().top == 3` 与面板高 492（9 行）。


## 候选拼音显示开关（T-127）

用户指令："设置里面可以设置候选框是否显示拼音及声调" —— 候选窗拼音行
（含声调）的显示开关。

决策与形态（沿用 FR-023/FR-024 开关先例，T-103）：

- **配置格式**：`ConfigFile.candidate_show_pin: bool`，缺省 `true`
  （保持历史行为；旧配置无该字段按开加载 —— 宽松解析、不递增
  `CONFIG_FORMAT_VERSION`，与 `enable_abbreviation`/`enable_fuzzy` 同模式）。
- **装配项，与主题同口径**：TSF 侧在装配期读一次（`configured_candidate_window`），
  设置重启输入法后生效 —— 设置窗口如实提示，不假装即时生效。
- **接线**：`CandidateWindow`/`CandidateWindowOptions`/`CandidateWindowState`
  三层携带 `show_pin`；paint 短路条件为
  `self.show_pin && !pin_text.is_empty() && pin_text != base`。关闭后候选行
  走既有无拼音绘制路径（主文本与序号整行垂直居中）—— 无新增布局计算，
  译文层拼音抑制与长拼音行为不受影响。新增构造
  `with_theme_and_show_pin`/`with_custom_theme_and_show_pin`，旧构造保持
  默认开；demo 增 `--no-pin` 便于截图验收。
- **设置 UI**：常用设置页新增「候选拼音」条目（自定义主题与英文输入法
  之间），关闭/显示二选一控件复用主题/模式/在线更新同一套 chips 机制
  （`ItemControl::CandidatePin`/`ChipValue::CandidatePin`/
  `candidate_pin_chips`）；`load_/save_candidate_show_pin` 与所有单字段
  写入方一致遵循 S-8 提交前重读。

验证：core（serde 默认开 + 往返）、settings（状态装载、chips 几何、
持久化保留字段）、ime 套件单测全绿；96dpi demo 截图像素探针 ——
`--no-pin` 拼音带（y117-127）灰像素 0 对默认 25（拼音行确实消失），
主文本带两态均正常。

## 部署通道备忘（流程、电池供电）

用户机部署通道是计划任务 `ZhuYeImeElevated`（`InteractiveToken` +
`RunLevel HighestAvailable` + `DisallowStartIfOnBatteries=true`）。笔记本
**电池供电**时该任务永远 `Queued`—— 两次验证（21:53 与 21:58）Last Result 0、
日志不动。采用的工作区：`Start-Process powershell -Verb RunAs -File
<tsf>\elevated-worker.ps1`（一次 UAC 确认）—— worker 的单次请求/响应模型
直接跑提权进程即可，无需计划任务。想靠 `schtasks /Create /XML` 改掉电池
限制在 Medium IL 下被拒，所以下次部署记住：UAC 直跑或插电源二选一。
