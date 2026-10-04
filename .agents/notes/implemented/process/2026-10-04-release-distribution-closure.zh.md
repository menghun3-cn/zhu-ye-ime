# Agent Note: 发布分发闭环与发布工具链（P-01，T-094）

Status: implemented

[English](2026-10-04-release-distribution-closure.md) | 中文

## 问题

M6-U（[词典更新信任链](../architecture/2026-09-29-dictionary-update-trust-chain.zh.md)）
交付了"签名 manifest + 内置公钥验签"的信任机制，但没有把"代码 → 用户机器"的
正式通道接通，四处缺口在 v0.1.1-alpha（2026-10-04，首个含更新器的引擎）发布后
尤其突出：

1. **无密钥生成工具**：`sign-manifest` 要求 `ZHU_YE_RELEASE_SECRET_KEY`，但仓库
   没有任何工具或手册生成这枚私钥/公钥——首次发布无从准备密钥材料。
2. **release 构建从不注入公钥**：`zhu-ye-updater` 的公钥经 `option_env!` 注入，
   而构建脚本从未设置 `ZHU_YE_RELEASE_PUBLIC_KEY`。未配置时更新器在内置公钥
   缺失时**拒绝一切更新**，等于 v0.1.1-alpha 的更新器实际不可用。
3. **无发布资产组装与端到端验证**：可分发包、签名 manifest、安装 zip 的发布
   产物组装没有脚本，也没有开工前的可重复全链验证。
4. **无发布操作手册**：密钥管理、Git 发布、资产上传、更新源验证散落各处，
   无法交给操作者按步执行。

## 决策

### `zhu-ye-dict keygen`：发布密钥对生成

`zhu_ye_core::generate_keypair()`（`manifest.rs`）用 `getrandom`（系统熵源，
跨平台，不引入 rand/Windows 专有 API）生成 32 字节种子，派生 ed25519 公钥；
输出 `(私钥 hex, 公钥 hex)`。私钥只在发布环境保存（见 `docs/发布流程.md`），
**绝不入库**；公钥可入库。新增 CLI 子命令 + 单测（派生一致、两次调用不同）。

### `scripts/assemble-release.ps1`：发布资产组装

从根 `Cargo.toml` 读版本；强制 `cargo clean -p zhu-ye-updater` 后构建——
`option_env!` 的公钥变化**不会被 cargo 增量缓存感知**，不 clean 可能把旧公钥
编进发行产物（本 note 首版手工验证即踩中一次）。组装 `target/release-assets/<v>/`：

- `packs/{it,med,slang}.zyct`：仅分发包进 manifest（`DISTRIBUTABLE_PACK_IDS`）；
  `base.zyct` 随安装包只读交付、`en.zyen` 是引擎资产，都不进更新源；
- `manifest.json`：`build-manifest`（包版本 = 发布日，`min_engine 0.1.1`）→
  `sign-manifest` → `verify-manifest`（发布检查清单 7.4 的逐包哈希/大小复核）；
- `ai-zhu-ye-ime-<v>.zip`（复用 `package-portable.ps1` 产物改名）；
- `SHA256SUMS.txt`（上传核对清单，供 `gh release upload`）。

### `scripts/verify-release-e2e.ps1`：发布前全链验证

用 `curl` 的 `file://` 协议走完整链路（不含真实网络；TLS 传输由上游提供，本地
验证的是"拉取 → 内置公钥验签 → 下载 → 哈希/大小校验 → staging+rename 原子落地 →
幂等"）：① `apply` 落地全部可分发；② 落地包与 manifest 哈希/大小一致；③ 篡改
落地包被检测并重新拉取自愈；④ 篡改 manifest 被内置公钥拒绝。

### `docs/发布流程.md`：发布操作手册

步骤 1-6（git-publish → 注入公钥构建/组装 → 上传 Releases → 更新源验证 →
发布清单收尾 → 同步回 develop）+ 密钥管理（生成/存放/轮换/泄露预案）+
故障预案表。

### 版本门槛修复（e2e 发现的关键缺陷）

`version_at_least` 原实现对每段 `parse::<u64>()`，`0.1.2-alpha` 的 `2-alpha`
解析失败按 0 处理 → 客户端引擎版本被当作 `0.1.0`，永远低于 `min_engine 0.1.1`，
**以 v0.1.2-alpha 定版的发布会被自己的更新器拒之门外**。修复为每段取前导数字
（`2-alpha` → 2），保留"非数字段按 0、缺段按 0 补齐"的既有语义；补 6 条
pre-release 断言。

### 正式发布密钥配置 GitHub（T-095）

2026-10-04 以 `zhu-ye-dict keygen` 重新生成正式密钥对（替代并作废旧测试密钥对）：
私钥仅存于 GitHub Actions secret `ZHU_YE_RELEASE_SECRET_KEY`（stdin 写入，不落盘、
不出现于进程参数与仓库），公钥为 Actions variable `ZHU_YE_RELEASE_PUBLIC_KEY`
（= `832dc4a549afed41b013b6aae8ebc7739b88d6fde9a7831336e9d59231067cce`，公开材料）。
镜像密钥给其他维护者的安排留待后续（需求 L235）。

### `.github/workflows/release.yml`：CI 化正式打包（T-095）

推送 `v*` tag 自动执行全链；`workflow_dispatch` 手动跑同一全链（`upload` 输入为
true 才上传），`concurrency` 组防止并发发布构建相互覆盖。windows-latest 步骤：
checkout → rust-toolchain → rust-cache → `fetch-sources.ps1`（pins 锁定校验下载，
含 bundle 抓取）→ `unpack-cedict.ps1` → `build-en-wordbook.ps1`（en.zyen）→
`build-base` / `build-pack it|med` / `build-slang` → `assemble-release.ps1`（公钥
注入构建、私钥签名 manifest、verify 复核、zip、SHA256SUMS）→
`verify-release-e2e.ps1`（4/4）→ `gh release create/upload`（6 项资产，仅 tag
推送或 `upload=true` 时）。secret 以环境变量进 runner，不出现于 `run` 文本。

### CI 试跑暴露的干净检出加固（T-095）

试跑暴露了一批只在本地工作区成立的假设（`data/raw`、`data/artifacts` 由早前
手工构建产生），逐一修复：

- `fetch-sources.ps1`：`Download-Once` 写入前创建目标目录（git 不跟踪空目录，
  干净检出下 `data/raw` 不存在）；
- `zhu-ye-dict` `en-build`：写 `data/artifacts/en.zyen` 前创建父目录（唯一缺
  `create_dir_all` 的构建输出；base/pack/slang 均已具备）；
- `data/pins/ecdict.json`：url 笔误 `ecdict-full.csv` → `ecdict.csv`（仓库实际
  文件名；内容哈希不变，无需重锁）；
- `data/pins/social-media-zh.json`：kind 由 `url`（误取到仓库主页 HTML）修正为
  `bundle` + `fetch_script`（既有的合并脚本），SHA-256 锁定不变；
- `scripts/unpack-cedict.ps1`：把锁定的 `data/raw/cedict.ts.gz` 幂等解压为构建
  链读取的明文 `cedict_ts.u8`，并加入 workflow 步骤。

### CC-CEDICT 月度更新重锁（T-095）

试跑按预期拦截 CEDICT 源漂移（pin 锁定 2026-09-21 快照，mdbg 月度更新）。
人工核验新内容（gzip 完整、9,858,607 字节 / 约 12.5 万词条、抽查含新增网络词）
后于 2026-10-05 重锁（`850243FB…` → `FD16B26F…`，size 3974945 → 3978521）。
CI 发布产物据此使用新 CEDICT 构建；本地开发锚（既有 zyct 产物与 T-057 eval
基线）在发布检查清单回归复跑前保持不变。

## 曾考虑的替代方案

- **keygen 用 PowerShell/openssl 实现**：否决——Rust 无新依赖（`getrandom` 已跨
  平台且是熵源标准）、可单测、与其余 `zhu-ye-dict` 子命令统一。
- **e2e 用本地 HTTP 服务**：否决——`file://` 经 `curl.exe` 验证同一下载抽象，
  避免端口/URL ACL/杀软噪音；真实 HTTPS 路径由 GitHub Releases 提供，发布后
  在真实环境下验证（VM 恢复后补）。
- **发布 zip 直接沿用 `package-portable.ps1` 的 `-test` 名字**：否决——发布
  资产用正式名，测试包名留给内部构造。

## 后果

- 发布动作现在 = 推送 `v*` tag → `release.yml` 构建、组装、以正式密钥签名、
  e2e 验证并上传全部资产到 GitHub Release（版本含 `-` 时为 prerelease）；手动
  全链试跑一键 `workflow_dispatch`（默认 `upload=false`）。**v0.1.2-alpha 起更新器
  才真正可用**（先前产物内置空公钥，拒更）。
- 正式密钥对已配置 GitHub（secret + variable）并经 CI 试跑端到端证明（内置公钥
  `832dc4…`、签名 manifest、e2e 4/4）；旧测试密钥对作废，不得签署发布产物。
- 换钥成本高（信任锚编译内置，需随引擎发布），列入发布手册故障预案。
- 本 note 是流程层记录，不取代信任链机制 note（`2026-09-29-dictionary-update-trust-chain`，
  保持 active，交叉引用）。
- 验证证据：CI 试跑 37242703873 全链 5m20s 通过（pins 15/15 锁定、en.zyen +
  base/it/med/slang 构建完成、正式密钥签名并复核、e2e 4/4 全绿）；五轮试跑收敛
  （fetch 目录 → ecdict url → social kind → 解压 → CEDICT 重锁）。
