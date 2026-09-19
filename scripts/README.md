# 项目脚本

本目录存放竹叶输入法开发流程脚本，全部面向 Windows/PowerShell 与 Rust/Cargo/git，不依赖 Node、pnpm、Vitest 或 TypeScript。

## 校验脚本

| 脚本 | 用途 |
| --- | --- |
| `verify-agent-notes.ps1` | 校验 Agent Notes 目录结构、文件格式、归档三件套与 `manifest.json` 封存；`-ArchiveWrite` 追加封存新归档笔记 |
| `verify-translation-pairs.ps1` | 校验 `.md` / `.zh.md` 双语配对的 `.i18n.yaml` 一致性记录；确认一致后可用 `-Write` 重写记录 |

## 常用命令

```powershell
# 构建 v1 词典数据包（默认 data/artifacts/seed.zyct）
cargo run -p zhu-ye-dict -- build

# 检查与完整性校验
cargo run -p zhu-ye-dict -- inspect data/artifacts/seed.zyct
cargo run -p zhu-ye-dict -- verify data/artifacts/seed.zyct

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

原 TypeScript/pnpm 版脚本已移除；需要自动化完整文档站、翻译简报等扩展门禁时，后续以 Rust/Cargo 或 PowerShell 等价实现。
