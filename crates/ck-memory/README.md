# ck-memory

## Overview

Domain-logic crate holding the session-memory retention vocabulary of Context Kernel. Unlike the storage and search adapters, it depends on no other workspace crate and performs no I/O; it defines the retention classes for session event kinds, the retention policy that decides which events survive, and the recall-scope enum. Its contracts are consumed by the session-memory retention paths that run against the `session_events` store.

## Subdirectories & Boundaries

- src: [./src/README.md](./src/README.md) - Component contract for `lib.rs`: `EventRetentionClass`, `classify_event_kind`, `SessionRetentionPolicy` (with `should_preserve_event`), and `MemoryRecallScope`; pure logic with no side effects.

## Verification

- Run: `cargo test -p ck-memory`
