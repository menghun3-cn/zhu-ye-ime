# 竹叶输入法（ai-zhu-ye-ime）

一款轻量的 Windows 中文输入法：全拼输入，候选区直接显示中英译文，按 `Tab` 即可上屏译文。目标是干净、快速、无广告，核心算法自研，本地优先，并为 AI 翻译/润色/联想/预判/建议预留可插拔接口。

当前状态：M1-M5 功能已完成开发与自动化验证，包含 TSF 注册/卸载、全拼输入、候选窗、键位交互、本地双语译文层、性能脚本、安装/便携包与 AI 接口预留；T-010 至 T-013 等待虚拟机/真机端到端验收。详细安装与使用见 [安装与使用指南](docs/安装与使用.md)。

## 特性

- Windows 10/11 x64，基于 TSF 的现代输入法
- 中文全拼输入，静态模型 + 上下文 + 用户词学习排序
- 中文候选行内附英文译文，`Tab` 切换译文层
- 默认完全离线，零网络请求；后续 AI 能力默认关闭、显式开启
- 词典 mmap 只读加载，冷启动快，无后台常驻进程
- Rust workspace 分层：算法核心与 Windows 适配严格分离

## 目录结构

```text
docs/                          需求、设计、验收、架构、任务清单等文档
crates/zhu-ye-core/            纯 Rust 算法核心（拼音、词典、排序、翻译、AI 接口）
crates/zhu-ye-ime/             TSF 输入法 DLL 与候选窗（Windows 适配）
crates/zhu-ye-dict/            词典数据管线构建器
crates/zhu-ye-cli/             自检、基准、候选演示 CLI
scripts/                       安装/卸载与数据脚本
```

## 构建环境

- Windows 10/11 x64
- Rust 1.98.1+（`rust-toolchain.toml` 固定 1.98.1），工具链 `x86_64-pc-windows-msvc`
- Git

## 快速开始

```powershell
cargo build --workspace
cargo test --workspace
cargo clippy --all-targets -- -D warnings
cargo run -p zhu-ye-cli -- demo nihao
```

TSF 安装/卸载脚本（需要管理员 PowerShell）：

```powershell
.\scripts\install.ps1
.\scripts\uninstall.ps1
```

无需管理员即可校验 DLL 导出：

```powershell
.\scripts\verify-tsf-dll.ps1 -DllPath .\target\release\zhu_ye_ime.dll
```

生成便携测试包（复制到其他 Windows 机器测试，无需安装 Rust）：

```powershell
.\scripts\package-portable.ps1
```

性能基准与阈值验收（正式验收加 `-Release`）：

```powershell
cargo run -p zhu-ye-cli -- bench
.\scripts\bench.ps1 -Release
```

安装、启用、键位、用户词管理、升级与卸载请阅读 [安装与使用指南](docs/安装与使用.md)。

## 文档索引

- [需求规格说明书](docs/需求规格说明书.md)
- [方案设计](docs/方案设计.md)
- [验收标准](docs/验收标准.md)
- [架构文档](docs/architecture.md)
- [任务清单](docs/todos-list.md)
- [任务归档](docs/todos-done.md)
- [数据许可证](docs/licenses.md)
- [安装与使用指南](docs/安装与使用.md)
- [变更记录](CHANGELOG.md)

## 路线图

- M0：文档基线 + workspace 脚手架
- M1：TSF 最小闭环（注册、上屏、卸载）
- M2：拼音核心 + 词典管线
- M3：候选窗 + 完整交互 + 用户词
- M4：双语翻译层 + 译文切换
- M5：性能、安装、文档、AI 接口
- M6：词典体系与自动更新（多包组合、网络语缩写、更新链路）
- M7：输入体验优化（简拼/首字母、模糊音纠错、整句 Beam Search）

## 许可证

项目代码许可证为 `MIT OR Apache-2.0`（见根 Cargo.toml）。第三方数据来源与许可证见 [docs/licenses.md](docs/licenses.md)，变更记录见 [CHANGELOG.md](CHANGELOG.md)。
