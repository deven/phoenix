# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Phoenix is a Computer-Mediated Communication (CMC) system — a TELNET-based chat/conferencing server with
continuous history since 1992 (C original, C++ from 1993, now being ported to Rust as `phoenix-cmc` v2.0.0).
It is strictly a CMC in the CONNECT lineage, **not** a MUD/MOO. Signature features: server-side remote echo,
Emacs-style line editing on ANSI terminals, input redrawing during asynchronous output, detach/attach session
persistence, and TIMING-MARK-based delivery confirmation.

**The prime directive: parity first.** v2.0.0 must be a drop-in replacement for C++ v1.0.0 — a faithful,
side-by-side-readable translation of the C++ structure and behavior, comments included. Deviations are permitted
only for language necessity, Tokio's async requirements, or the small set of declared innovations: Argon2
password hashing (replacing `crypt()`), UTF-8 (replacing Latin-1, character-for-character), async file I/O,
in-server password management, and foreground-only operation by default (with a `--daemonize` option planned).
Before flagging any behavior as a bug or deviation, **cross-check the C++ semantics first** — surprising behavior
is often faithful behavior. Architectural rulings are Deven's alone: analyze, propose, and implement, but never
decide unilaterally.

## Repository Layout

- `phoenix-cmc/` — **the active Rust port** (v2.0.0 work happens here)
- `phoenix_cmc/` — retired earlier Rust crate (pre-port experiments: actor macros, `config!`, event system);
  reference only, do not modify
- `C/`, `C++/` — the historical C (1992–93) and C++ (1993–) implementations; `C++/` is the parity reference
- `client/` — the original custom client (CONNECT-derived, TELNET-based)
- `README`, `HISTORY`, `TODO`, `PORTING`, etc. — historical project documents; the Git history itself is a
  forensically reconstructed archive (892+ commits) and must be treated with archival care

## Build and Development Commands

All Rust work happens in `phoenix-cmc/` (edition 2024, `rust-version = 1.85`).

- `cargo build` / `cargo build --release` — build (release uses LTO, single codegen unit)
- `cargo run -- [--cron] [--debug] [--port 9999]` — run the server (default port 9999; `--cron` exits quietly if
  the port is busy; `--debug` enables debug behavior and foreground logging)
- `cargo fmt` — format (see `rustfmt.toml`: `max_width = 160`, small-heuristics Max, Unix newlines)
- `cargo clippy --all-targets` — lint
- `cargo test` — run tests

**Run `cargo fmt`, `cargo clippy --all-targets`, and `cargo test` before considering any change complete.**

At startup the server creates and changes into its lib directory (`phoenix/` under the working directory) and
creates a `logs/` subdirectory. Before starting a test server, kill any stale server process and confirm it is
gone — a stale server may keep serving while the new one errors out.

## Architecture

Hybrid actor model with lock-free shared reads:

- **Actors.** Each subsystem is an actor: a `*Obj` struct owned by its Tokio task (`ServerObj`, `SessionObj`,
  `TelnetObj`, `DiscussionObj`, `UserManagerObj`), driven by a `*Msg` enum over an **unbounded** mpsc channel.
  Unbounded is an architectural ruling, not an oversight: backpressure is wrong for fan-out (one stuck consumer
  would stall healthy members), bounded channels reintroduce deadlock through send-as-wait, and lossy `try_send`
  corrupts the control plane. Memory governance belongs at the spool/retention layer, not the transport.
- **Handles.** Each actor has a cheap clonable handle `Foo(Arc<FooInner>)`. Naming rule: `*Inner` holds
  cross-task-readable state (atomics, arc-swap fields) plus the message sender; `*Obj` holds actor-task-private
  state. State that only the actor task touches belongs in the `*Obj`, not as an atomic in the `*Inner`.
- **Shared reads.** Global registries are `arc-swap`-based `AtomicHashMap`s over `im` immutable maps (RCU
  pattern): `SESSIONS`, `DISCUSSIONS`, `USERS`, `USER_MANAGER`, `DEFAULTS` (all `LazyLock` statics). Reads are
  lock-free snapshots; mutations serialize through the owning actor.
- **Stratified waiting.** Session actors may await non-session actors using **oneshot** reply channels;
  non-session actors never await anyone. Oneshot per request (not persistent mpsc) because persistent mpsc
  cannot detect peer death.
- **`repr_u8_enum!`** (`atomic.rs`) generates atomic-backed enums with exhaustive `From<u8>` — new variants
  become compile errors instead of silently defaulting. Use it for any enum stored atomically (e.g.,
  `LoginState`).

Modules: `server`, `session`, `telnet` (full TELNET protocol implementation), `discussion`, `user`, `sendlist`,
`output`, `text` (case-insensitive `Text` type), `name`, `timestamp`, `atomic`, `constants`.

## Code Conventions

- 4-space indentation; `max_width = 160`; Deven rewraps comments at 120 columns — never anchor diffs on comment
  lines, use code-line anchors only
- `#[async_backtrace::framed]` on async functions
- Wrapper types for domain safety (`Name`, `Text`); `anyhow::Result` at boundaries, `thiserror` for typed errors
- `Debug` impls destructure the struct in the impl body so missing fields become compile errors; do not use
  `finish_non_exhaustive()` to paper over fields
- Logging via `log` + `env_logger` (`RUST_LOG=debug cargo run`); the `println!("=== DEBUG: ...")` scaffolding is
  transitional and slated for conversion to `debug!`/`trace!`
- Feature flag `guest-access` is **enabled by default** (`default = ["guest-access"]`); build with
  `--no-default-features` to exclude it

## Commit Conventions

- One commit per logical fix
- Imperative subject line; detailed body with root cause, C++ correspondence, reproduction details, and
  verification performed — full bodies, not just subject lines
- AI-assisted commits carry a model attribution in a parenthetical; commits without one are Deven's own work.
  Attribute design ideas accurately — do not credit a model for decisions that originated with Deven

## Working Against the C++ Reference

- `C++/` is the behavioral ground truth for parity questions; when in doubt, read it before proposing a change
- **Warning:** commit `bcfa2fa` (2019 postfix-iterator refactor) broke all `while (u++)` login lookups in the
  C++ codebase. Revert it before testing C++ behavior: `git show bcfa2fa | patch -p1 -R`
- The gold standard for parity disputes is empirical byte-stream verification against the original binary
  (a Python TELNET scripting client has been used for this); "reads equivalent" is not proof
