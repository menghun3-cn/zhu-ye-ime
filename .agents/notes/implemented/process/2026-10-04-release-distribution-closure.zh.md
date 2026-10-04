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

## 曾考虑的替代方案

- **keygen 用 PowerShell/openssl 实现**：否决——Rust 无新依赖（`getrandom` 已跨
  平台且是熵源标准）、可单测、与其余 `zhu-ye-dict` 子命令统一。
- **e2e 用本地 HTTP 服务**：否决——`file://` 经 `curl.exe` 验证同一下载抽象，
  避免端口/URL ACL/杀软噪音；真实 HTTPS 路径由 GitHub Releases 提供，发布后
  在真实环境下验证（VM 恢复后补）。
- **发布 zip 直接沿用 `package-portable.ps1` 的 `-test` 名字**：否决——发布
  资产用正式名，测试包名留给内部构造。

## 后果

- 发布 = 生成/载入密钥 → 注入公钥构建 → `assemble-release` → `gh release
  create/upload` → `verify-release-e2e`；**v0.1.2-alpha 起更新器才真正可用**
  （先前产物内置空公钥，拒更）。
- 换钥成本高（信任锚编译内置，需随引擎发布），列入发布手册故障预案。
- 本 note 是流程层记录，不取代信任链机制 note（`2026-09-29-dictionary-update-trust-chain`，
  保持 active，交叉引用）。
- 测试密钥对仅用于本机管线验证，**发布前重新 `keygen`**，测试密钥不得再用。
- 验证证据：`assemble-release` 全流程通过（ed25519 签名 + verify 3/3）；e2e 脚本
  4/4 全绿；`cargo test` 291 项（含新增 keygen 与版本门槛断言）。
