# Agent Note: User dictionary persistence and recovery for T-009

Status: implemented

[English](2026-09-19-user-dict-persistence.md) | 中文

## 问题

FR-003 要求把用户选择的词在本地持久化，并支持删除词与重置用户词库；NFR-002 要求用户词库损坏时自动备份恢复。此前 `UserDictionary` 只持有内存状态：选择候选后从不写盘，学习到的词会在进程退出后丢失，而且 IME 与 CLI 之间没有可共享的持久化契约（路径、格式与失败语义）。

## 决策

`zhu-ye-core::user_store::UserDictStore` 负责磁盘契约。调用方注入 JSON 文件路径：TSF 宿主使用 `%APPDATA%\zhu-ye-ime\user_words.json`，CLI 与测试使用各自路径，核心库保持平台无关。

文件采用版本化 JSON：`{"version": 1, "entries": [{"word", "pinyin", "frequency", "last_used"}]}`。`UserWord` 派生 serde；`UserDictionary` 新增 `from_entries`，过滤空词文本、空拼音与零词频条目。`words_sorted` 现在按词频降序、再按词与拼音定序，保证落盘文件确定。

写入是原子的：`save` 创建父目录、写 `.tmp` 文件、调用 `sync_all`，再重命名覆盖目标；失败时清理临时文件。`load` 把文件不存在视为空库；解析失败与不支持的旧版本会把损坏文件改名为 `.bak` 并保存空库；高于当前支持的版本则拒绝打开且不触碰原文件，避免软件降级破坏用户数据。

`InputEngine` 新增 `with_user_store`，`Candidate` 携带学习所需的拼音；空格或数字选择真实候选提交时记录选择，并经由 store 落盘。回车与拼音原文回退不学习。`delete_user_word` 与 `reset_user_words` 同时更新内存与磁盘。CLI 提供 `user list / delete / reset` 子命令用于可观测验证。

## 曾考虑的替代方案

**异步保存每次选择。** 落选：文件很小，同步写入足够快；同步保存让持久化契约容易测试与审查。若慢盘上出现延迟，再并入 T-017 观察。

**把用户词放进词典二进制。** 落选：词典文件是只读 mmap 数据，用户词是可变的本机状态；混在一起会破坏 T-006 的替换边界，也让备份与重置不安全。

**使用二进制或 SQLite 格式。** 落选：数据量小且属用户所有；可读的版本化 JSON 便于排查，serde_json 也符合既定的用户态持久化选型。

**原地覆盖损坏文件。** 落选：NFR-002 要求可恢复与可诊断，必须保留归档；`.bak` 至少保留最近一份损坏文件。

## 后果

选择记录现在能跨进程存活，并会在下次启动时参与排序。`Candidate` 新增拼音字段，为提交提供了持久化的学习键，也为后续同步与 AI 功能留了入口。TSF 宿主接线后，DLL 只在 `%APPDATA%` 下写入；测试与 CLI 未配置时不会触碰该路径。

T-009 保持“进行中”，直到 VM 端到端复核通过（安装、输入、选择、重启，再观察排序与 JSON 文件）。[2026-09-19-candidate-ranking-static-model.md](../feature/2026-09-19-candidate-ranking-static-model.md) 中的排序接口无需结构改动即可消费持久化词库。
