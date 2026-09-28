# Agent Note: Candidate window blue-frame/blue-text/red-selection palette

Status: implemented

[中文](2026-09-25-candidate-window-blue-red-palette.zh.md) | English

## Problem

After T-028/T-030 the default light palette was the Sogou-classic look:
white background, light-gray rounded border, near-black candidate text, a
light-blue selected block with deep-blue selected text. The user directive:
the candidate window should have a **blue border**, **blue candidate text**,
and a **red selected row**.

## Decision

- **Light (default):** white background `#FFFFFF`; rounded border
  `#1E88E5` (bright blue, same as the candidate text); candidate text
  `#1E88E5`; selected block stays `#E6F2FE` (light blue) with **selected text
  red `#D32F2F`** as confirmed by the user (red text + light-blue block, not
  a red block); markers and translations stay secondary gray `#999999`.
- **Dark (explicit-only, isomorphic):** dark background `#202020`; border
  `#42A5F5`; candidate text `#64B5F6`; selected block unchanged `#3A4A5C`
  with bright-red selected text `#FF8A80`; markers/translations `#9E9E9E`.
- The light default and the dark-only-via-explicit policy of T-030 are
  unchanged; only the palette values moved.
- FR-009 皮肤色板 acceptance row updated with the exact values and the
  T-032 tag.

## Alternatives considered

**Red selected block (red background, white or red text).**
Rejected per user choice: red text on the existing light-blue block keeps the
T-028 block structure and reads best.

**Dark-blue border with bright-blue text as two tones.**
Rejected per user choice: "亮蓝" applies to both border and candidate text.

## Consequences

Screenshots and pixel histograms from the VM drill on v0.1.3 (shots9) are the
verification artifact: border pixels `1E88E5`, first-row (selected) text
`D32F2F`, remaining rows `1E88E5`. The palette unit test locks the new
values. This supersedes the palette values referenced in
2026-09-25-candidate-window-light-default-and-empty-panel.md, which still
holds for the light-default and always-visible-panel decisions.
