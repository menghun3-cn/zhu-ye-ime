# Agent Note: AI service contract aligned with the design

Status: implemented

[中文](2026-09-21-ai-service-contract-and-offline-default.zh.md) | English

## Problem

The M5 scaffold exposed an `AiService` trait, but its `translate` method
lacked the `TranslationDirection` parameter specified in the design. That made
the extension point ambiguous for future remote/local AI backends and
inconsistent with the existing `Translator` contract.

## Decision

`zhu-ye-core::ai` now reuses `crate::translate::TranslationDirection`;
`AiService::translate` takes the text and the direction and returns `None`
for unsupported backends. `OfflineAiService` explicitly implements the full
trio (`suggest` empty, `translate` None in both directions, `polish` None) to
keep the default product zero-network. `TranslationDirection` is re-exported
from the core crate so out-of-crate services can implement the trait
ergonomically.

## Alternatives considered

**Keep the directionless `translate`.** Rejected: it silently diverges from
the design and cannot distinguish zh-to-en from en-to-zh for future AI
translation.

**Put direction into `InputContext` only.** Rejected: translation is a
standalone capability (`polish` also takes plain text), and making callers
wrap context adds schema without value.

## Consequences

The AI extension point now matches the design contract and can be implemented
by remote or local-model backends without rework. The offline default remains
empty and performs zero network calls; existing typing behavior is unchanged.
Unit tests pin the offline suggest/translate/polish behavior.
