# Agent Note：通讯录检索扩展到公司/地址/邮箱（T-102，D-20 修订）

Status: proposed

English | [中文](2026-10-05-contact-index-extra-fields.zh.md)

## Problem（问题）

通讯录检索当前仅按姓名建索引（决策 D-20，2026-10-02 确认，见 note
2026-10-02-contacts-scenario9）：输入 `zhangsan`/`zs` 可达联系人，但输入公司
名、地址或邮箱用户名永远不命中。用户点名清单批三明确要求"通讯录检索扩展
到公司/地址/邮箱"，即修订 D-20，也与场景 9 需求正文（§16.1"地址/邮箱/公司
名也可联想"）对齐。

## Proposal（方案）

1. **vCard 解析扩展三个检索字段**：`VCardContact` 增 `org`/`email`/`address`
   （另加 `new(name)` 构造，供测试与仅姓名联系人使用）。`extract_contact`
   收集 `ORG`/`EMAIL`/`ADR` 全部非空值按出现顺序以 `; ` 拼接（多值不丢），
   字段顺序无关（FN 可在其前/后，卡尾统一组装；FN 与 N 并存时 FN 为准），
   组前缀/转义处理沿用。`TEL` 等其余字段仍忽略（不在点名范围）。
2. **索引四源建键**：`build_contact_index` 改经 `contact_keys` 合并
   姓名 + 公司 + 地址 + 邮箱各自 `annotate_name` 结果后去重（跨源同键只索引
   一次）。中文公司名/地址注音建键（`竹叶科技`→`zhuyekeji`、`北京市`→
   `beijingshi`）；邮箱归一为连续小写键且符号跳过（`bob@acme.com`→
   `bobacmecom`），输入用户名段前缀即命中，`@` 后域名段永不独立成键。任意
   来源命中均上屏**姓名**。简拼键派生（FR-037）对合并后的每个键原样生效。
3. **边界纪律**：无新运行时状态、无配置、TSF 层零改动——仅解析输出与索引
   契约两个改动面，`contact_candidates` 不动。T-057 eval 不可能回退（联系人
   候选只在命中时出现）。

## Alternatives considered（备选方案）

- **每联系人单串合并建键**（`annotate(name + ' ' + org + …)` 一次注音）：
  省事但跨字段拼音串混杂、去重边界不清 → 放弃；按字段建键集再合并更清晰且
  确定。
- **多值用 `Vec<String>`**：忠实 vCard 多重性，但索引只需文本建键、展示与
  测试只断言拼接语义 → 单 `String` + `; ` 拼接保持最小面。
- **邮箱只做精确匹配**：对"输用户名段"的典型流程不友好 → 放弃；连续键前缀
  覆盖 `bob`/`bob@`/`bob@acme` 统一语义。
- **TEL 作为第五键源**：点名范围仅公司/地址/邮箱；号码检索价值低且扩大改动
  → 推迟，验收标准中仍标注 TEL 忽略。

## Acceptance criteria（验收标准）

- `cargo test --workspace` 全绿：`vcard` 22 项（含 6 项字段提取）、`contacts`
  20 项（含 6 项四源建键）；ime 与 host-e2e 改 `VCardContact::new` 构造仍全过。
- host-e2e `--m12` 9/9：新增用例 7（公司拼音 `zhuye`→张三）、8（邮箱用户名
  前缀 `bob`→张三）、9（地址拼音 `beijing`→张三）全 PASS；既有用例 1-6 不变。
- 文档：需求 §16（D-20 行带 2026-10-05 修订注）、验收 §12.1（四源检索行）、
  通讯录设计 §1/§3/§4/§5/§7 与实现一致。
- 门禁：fmt --check / clippy -D warnings / git diff --check /
  verify-agent-notes / verify-translation-pairs 全部干净。

## Risks（风险）

- **键污染**：新增键源使更多拼音前缀可能命中联系人——这正是功能本身；去重
  保持候选条数有界、顺序确定（BTreeMap + 稳定追加）。
- **邮箱键不可拆**：连续键 `bobacmecom` 查询时无法分割，只支持用户名段前缀
  查询；日后如需按域名检索要单独建索引（刻意的范围封顶）。
- **多值膨胀**：`; ` 拼接放大键长；受每源 64 键上限与 CONTACT_INDEX_CAP
  双重锚定，无无界增长。
- **vCard 差异**：部分导出器写 `ADR;TYPE=HOME:...` 参数——既有参数处理已
  覆盖；非 UTF-8 CHARSET 卡仍跳过（不变）。
