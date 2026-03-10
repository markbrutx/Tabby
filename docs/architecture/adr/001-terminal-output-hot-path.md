# ADR-001: Terminal Output Hot-Path Exemption

**Status:** Accepted
**Date:** 2026-03-10

## Decision

Terminal output data bypasses `RuntimeObservationReceiver::on_terminal_output_received` and is emitted directly from the PTY read thread to the frontend via a Tauri event (`TERMINAL_OUTPUT_RECEIVED_EVENT`).

## Context

The PTY read thread in `shell/pty.rs` reads terminal output in 8 KiB chunks at high frequency. This byte stream must reach the xterm.js renderer in the frontend with minimal latency to avoid visible lag during interactive terminal use.

`RuntimeObservationReceiver` is an application-layer callback trait that routes infrastructure observations (PTY exit, CWD change, browser URL change) through `RuntimeApplicationService`. These observations are low-frequency, discrete events that update domain state (the `RuntimeRegistry`).

Terminal output is fundamentally different:

1. **Volume**: hundreds to thousands of events per second during heavy output (e.g., `cat` of a large file, build logs, scrollback flood).
2. **No domain state change**: raw terminal bytes do not update `RuntimeRegistry` or any domain aggregate. They are a passthrough to the renderer.
3. **Latency sensitivity**: each additional hop (trait dispatch, lock acquisition, re-emit) adds measurable delay that degrades the interactive terminal experience.

Routing terminal output through the trait would add overhead (trait dispatch, potential `Mutex` contention in the receiver, re-serialization) without any domain benefit.

## Consequences

- The PTY read thread emits `TERMINAL_OUTPUT_RECEIVED_EVENT` directly via `AppHandle::emit` (see `shell/pty.rs`).
- `RuntimeObservationReceiver::on_terminal_output_received` exists on the trait but is **not wired** to the PTY read thread. It is intentionally a no-op in `RuntimeApplicationService`.
- This is an **acknowledged design decision**, not an oversight.

## Future: OSC Sequence Detection

The trait method `on_terminal_output_received` is reserved for a future use case: **OSC sequence detection**. When shell integration is implemented, the PTY read thread may parse terminal output for OSC escape sequences (e.g., OSC 7 for CWD reporting, OSC 133 for prompt/command markers). Detected sequences would be forwarded to `RuntimeObservationReceiver` as structured domain events, while the raw byte stream continues to flow directly to the frontend.
