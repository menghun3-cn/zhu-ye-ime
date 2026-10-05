# Agent Note：简拼/模糊音关闭开关（T-103，O-05 修订）

Status: proposed

English | [中文](2026-10-05-abbreviation-fuzzy-switches.zh.md)

## Problem（问题）

决策 O-05（2026-10-02 确认，见 note 2026-10-02-ime-experience-optimization）为
"简拼与模糊音默认开启、不新增配置开关"。用户点名清单批三明确要求"简拼/模糊音
关闭开关"，即修订 O-05：觉得首字母展开或模糊纠错过于激进的使用者，需要能分别
关闭而不放弃输入法。

## Proposal（方案）

1. **配置**（`zhu-ye-core/pack_config.rs` `ConfigFile`）：新增布尔
   `enable_abbreviation` 与 `enable_fuzzy`，经 serde `default = "default_..."`
   缺省 `true`（与 `enable_domain_boost` 同款宽松模式）。旧配置缺字段按开启
   加载；**不递增 `CONFIG_FORMAT_VERSION`**（纯增量可选字段，T-073 理由）。
2. **引擎**（`zhu-ye-ime/input.rs`）：`InputEngine` 增两字段与
   `with_abbreviation(bool)` / `with_fuzzy(bool)` builder（镜像
   `with_domain_boost`）。两处候选路径门：
   - 简拼关 → FR-023 首字母展开块（主候选空 + 2-4 位纯小写 + 不可切分）不再
     执行；
   - 模糊音关 → FR-024 纠错组（模糊替换 + 少字母补全两类）整体不生成。
3. **装配**（`zhu-ye-ime/tsf.rs`）：引擎创建链在 `domain_engine` 旁追加
   `with_abbreviation(config.enable_abbreviation)` /
   `with_fuzzy(config.enable_fuzzy)`。
4. **刻意边界**（文档化，不随开关）：
   - 网络语缩写路径（FR-016/FR-017，M6-R）不变；
   - 联系人索引原生简拼键（FR-037）不变——它们是显式高置信数据存储的一等
     索引键，不属于词典缩写解码层；
   - 本批不做设置窗口 GUI（与 `enable_domain_boost` 同级：手改
     `config.json`；界面化留作后续增强）。

## Alternatives considered（备选方案）

- **合并单一"简拼/模糊音"开关**：比用户意图粗（"关闭开关"暗示两者各自存在）；
  两个独立布尔零成本。
- **模糊音关 = 变体上限置 0**：可行但隐藏意图、耦合内部常量；显式
  `enable_fuzzy` 条件自文档化。
- **设置窗口条目**：点名项说的是开关而非 GUI；与 `enable_domain_boost`
  一致（仅配置文件）且避免扩大本批范围（lsg）——推迟，设置窗口设计 §5.1
  已注明。

## Acceptance criteria（验收标准）

- workspace 全绿：core `pack_config` 27（含新开关往返/旧配置/单开关独立测试）、
  ime lib 191（含 5 项开关测试：nh 关闭不展开、整词不受影响、zongguo 不纠错、
  niha 不补全、联系人简拼键不受影响）。
- host-e2e：`--m7` 真实词典 27/27（含"关闭后 nh/zongguo 均不介入"）、
  `--m12` 9/9。
- 文档：需求 §12（O-05 行带 2026-10-05 修订注、FR-023/FR-024 开关行）、
  方案 §12.1 注 + 新增 §12.3.4、验收 §8（3 行）、架构 §10.1、
  设置窗口设计 §5.1（注明不做 GUI）。
- 门禁：fmt --check / clippy -D warnings / git diff --check /
  verify-agent-notes / verify-translation-pairs 全部干净。

## Risks（风险）

- **开关语义落差**：使用者可能以为开关也能关联系人简拼或 `[网络]` 缩写。
  已在需求 §12、方案 §12.3.4/§12.2 与本 note 显式写明边界；默认仍开启，未
  选择关闭前行为不变。
- **配置陈旧读取**：`enable_domain_boost` 在引擎装配时读一次（重启生效）；
  新开关复用该生命周期——沿用既有"重启输入法后生效"约定。
- **模糊音语义**：`enable_fuzzy=false` 同时关闭两类纠错（A 替换 + B 补全）。
  只要替换不要补全的细分粒度当前不必要（两者同属"打错/打快"纠错），设计
  文档已注明为当前范围。
