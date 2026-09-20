# Agent Note: Candidate window TSF integration and key interaction for T-013

Status: implemented

[中文](2026-09-20-candidate-window-tsf-integration.zh.md) | English

## Problem

FR-004, FR-006, and FR-013 require the candidate window to follow a live
composition instead of a static demo: the user must be able to page through
candidates, switch between Chinese and translation layers, toggle Chinese and
English modes, and select candidates with digit keys. T-012 delivered
rendering but explicitly deferred key routing, window positioning, and window
lifecycle wiring.

## Decision

The input engine owns paging and layer state in pure Rust, and the TSF adapter
owns key routing and the candidate window lifecycle.

`InputEngine` gains `CandidateLayer` (Chinese/Translation), a zero-based
`page`, and `page_size` (default 9, matching digit keys 1-9). `handle_letter`
and `handle_backspace` reset the page; `next_page`/`previous_page` wrap at the
ends. `toggle_translation_layer` only enters the translation layer when at
least one candidate has a non-empty translation, so the window can never show
an empty translation page. `commit` returns to the Chinese layer and records
the Chinese word as the learned user word even when the translation is what is
typed, while the committed text becomes `previous_word` for bigram context.

`candidate_ui_view` produces the snapshot consumed by the window. The view is
empty when the engine is in English mode even though the composition string is
kept, which hides the popup during English typing and restores it when mode
switches back.

TSF adds four key actions: Shift toggles mode (repeat key-down events with bit
30 of `lparam` are ignored so holding Shift does not flip mode repeatedly),
Tab toggles the layer, comma pages up, and period pages down. Tab and paging
are only consumed while a Chinese-mode composition is active; in English mode
they fall through to the host, as do letters, Backspace, Space, Enter, Esc,
and digits. Shift is always handled by the IME so it can recover from English
to Chinese mode.

Edit-session actions write text and then refresh the candidate window with a
placement computed from `ITfContextView::GetTextExt` on the composition range;
state-only actions refresh without an edit session and keep the window's
current position (`SWP_NOMOVE`). The window is hidden on commit/cancel,
composition termination, and `Deactivate`.

`CandidateWindow` is now a TSF-controlled popup: created, updated, hidden, and
destroyed on the host UI thread with no message loop of its own. Destroying it
does not call `PostQuitMessage`; the demo path keeps its own quit behavior.
The popup positions below the composition's text-ext bottom edge and clamps
into the monitor work area.

When Shift switches to English mode, the candidate window hides immediately,
and Tab/paging/selection no longer consume keys; switching back to Chinese
restores the previous composition and candidate layer.

## Testing

`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test --workspace` pass. New coverage checks paging/wrap, page-relative
selection and preview, page reset on input change, layer filtering, empty
translation-layer refusal, translation commit semantics and learning, mode
switch preserving composition, English-mode empty view, key classification for
Shift/Tab/comma/period, repeat suppression, and English-mode key fallthrough.
TSF lifecycle tests continue to pass: activate/deactivate, class factory,
aggregation rejection, and unload counter.

T-013 remains in progress until the build is installed on the VM and typing,
paging, layer switching, mode switching, translation commit, and popup
positioning are verified in Notepad.

## Alternatives considered

**Run the candidate window on its own thread.** Rejected: a separate message
pump would duplicate the TSF UI-thread contract, complicate COM apartment
rules, and add startup cost; the popup is lightweight and redraws in the host
thread.

**Query placement on every state-only refresh via a second edit session.**
Rejected: paging and layer toggles do not move the composition range, so
keeping the last position with `SWP_NOMOVE` avoids extra TSF requests and is
observably equivalent unless the host scrolls mid-composition.

**Hide the window but keep focus-stealing tool-window styles.**
Rejected: keeping `WS_EX_NOACTIVATE` is mandatory so the popup never steals
focus from the composition target.

**Commit translation and learn the translation text as the user word.**
Rejected: user-word frequency should reflect the Chinese word the user
chooses; learning the English output would pollute the Chinese ranking store
and hurt later sync/AI features.

**Page through the Chinese candidate list in English mode.**
Rejected: in English mode the composition is frozen by design and functional
keys should reach the application; the popup is hidden and key handling
returns to the host.

## Consequences

The M3 candidate window is now wired to live composition state: rendered
items, page, translation layer, and mode all come from one engine snapshot,
and commit/termination paths remove the popup promptly. The engine remains
platform-independent, so ranking, paging, and layer behavior are covered by
fast unit tests before any OS integration.

The main remaining risk is real-host behavior: `GetTextExt` coordinate
accuracy, popup timing inside synchronous edit sessions, and keyboard
interplay with Windows shortcuts still need VM verification. If the popup
position or refresh timing needs to be tuned, the `CandidateWindowPlacement`
boundary isolates that change from the engine and TSF key routing.
