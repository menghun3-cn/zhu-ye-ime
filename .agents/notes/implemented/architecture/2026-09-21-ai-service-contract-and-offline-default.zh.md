# Agent Note: AI service contract aligned with the design

Status: implemented

English | [中文](2026-09-21-ai-service-contract-and-offline-default.zh.md)

## 问题

M5 脚手架已经暴露 `AiService` trait，但其中的 `translate` 缺少方案设计规定的 `TranslationDirection` 参数，导致未来远程/本地 AI 后端的扩展点语义模糊，也与现有 `Translator` 合同不一致。

## 决策

`zhu-ye-core::ai` 现在复用 `crate::translate::TranslationDirection`；`AiService::translate` 接收文本与方向，未接入的后端返回 `None`。`OfflineAiService` 显式实现完整三项能力（`suggest` 空、`translate` 双向 `None`、`polish` `None`），保持默认产品零网络。`TranslationDirection` 从 core crate 重新导出，便于 crate 外的服务实现 trait。

## 曾考虑的替代方案

**保留无方向的 `translate`。** 否决：它与方案设计悄然偏离，未来 AI 翻译无法区分中英方向。

**只把方向放进 `InputContext`。** 否决：翻译是独立能力（`polish` 也只接收纯文本），让调用方包装上下文只会增加无意义的样板结构。

## 后果

AI 扩展点已与设计合同对齐，远程或本地模型后端可直接实现而无需返工。离线默认仍为空且零网络；既有输入行为不变。单测固定了离线 `suggest`/`translate`/`polish` 的行为。
