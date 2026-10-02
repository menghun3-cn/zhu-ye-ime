# 项目脚本

本目录存放竹叶输入法开发流程脚本，全部面向 Windows/PowerShell 与 Rust/Cargo/git，不依赖 Node、pnpm、Vitest 或 TypeScript。

## 校验脚本

| 脚本 | 用途 |
| --- | --- |
| `verify-agent-notes.ps1` | 校验 Agent Notes 目录结构、文件格式、归档三件套与 `manifest.json` 封存；`-ArchiveWrite` 追加封存新归档笔记 |
| `verify-translation-pairs.ps1` | 校验 `.md` / `.zh.md` 双语配对的 `.i18n.yaml` 一致性记录；确认一致后可用 `-Write` 重写记录 |
| `bench.ps1` | 构建/运行 `zhu-ye-cli bench` 并校验 `指标:` 行阈值；`-Release` 做正式验收，`-MaxUsPerOp` 覆盖阈值 |
| `package-portable.ps1` | 生成发行包（T-078）：`bin\`（TSF DLL + 设置窗口 + 更新器 + 基础词典）、`packs\`（三个领域包）、`scripts\`、`docs\`；干净机解压后直接 `install.ps1`，无源码树依赖 |
| `e2e.ps1` | 构建/运行 `host-e2e`，对种子词典执行核心输入闭环回归；存在真实词典时追加 smoke；`-Release` 正式验收，`-SkipBuild` 跳过构建，`-RealDictionaryPath` 指定真实词典 |

## 常用命令

```powershell
# 构建 v2 词典数据包（默认 data/artifacts/seed.zyct）
cargo run -p zhu-ye-dict -- build

# 检查与完整性校验
cargo run -p zhu-ye-dict -- inspect data/artifacts/seed.zyct
cargo run -p zhu-ye-dict -- verify data/artifacts/seed.zyct

```

```powershell
# 生成发行包（T-078：bin/ + packs/ + scripts/ + docs/）
.\scripts\package-portable.ps1

# 发行包模式安装（不构建、无源码树依赖）
.\scripts\install.ps1
```

```powershell
# Agent Notes 全量校验
.\scripts\verify-agent-notes.ps1

# 新归档笔记封存
.\scripts\verify-agent-notes.ps1 -ArchiveWrite

# 双语配对校验
.\scripts\verify-translation-pairs.ps1

# 双语配对确认一致后重写记录
.\scripts\verify-translation-pairs.ps1 -Write

# 提交/推送前 Rust 与 git 门禁
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
```

# 性能基准与阈值验收（正式验收加 -Release）
.\scripts\bench.ps1
.\scripts\bench.ps1 -Release -MaxUsPerOp 200
```

原 TypeScript/pnpm 版脚本已移除；需要自动化完整文档站、翻译简报等扩展门禁时，后续以 Rust/Cargo 或 PowerShell 等价实现。
