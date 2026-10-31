# ck-memory (src)

## Overview

This crate holds the session-memory retention vocabulary of Context Kernel as pure domain logic: a classification of event kinds into retention classes, a session retention policy, and a recall-scope enum. It has no dependency on `ck-core`, no I/O, and no storage or transport code; the declared dependencies (`anyhow`, `chrono`, `serde_json`) are not exercised by the current module. Consumers use it to decide which session events survive retention and to express recall scope.

## Interface

- [lib.rs:EventRetentionClass] - Enum with variants `Pinnable`, `Ephemeral`, and `Permanent`.
- [lib.rs:classify_event_kind] - Maps an event-kind string to an `EventRetentionClass` (mapping fixed in code; see Invariants).
- [lib.rs:SessionRetentionPolicy] - Policy struct with public fields `preserve_pinned` and `preserve_pinnable_kinds`.
- [lib.rs:SessionRetentionPolicy::default] - Default policy: `preserve_pinned = true`, `preserve_pinnable_kinds = false`.
- [lib.rs:SessionRetentionPolicy::should_preserve_event] - Decides whether an event is retained based on its pinned flag.
- [lib.rs:MemoryRecallScope] - Enum with variants `CurrentSession`, `RecentSessions`, and `AllSessions`.

## Invariants

- Kind classification is total and fixed: `"decision"`, `"architecture"`, `"requirement"`, and `"constraint"` classify as `Pinnable`; `"pinned"` and `"milestone"` classify as `Permanent`; every other kind classifies as `Ephemeral` [lib.rs:classify_event_kind].
- The default policy preserves pinned events only, with pinnable-kind preservation disabled unless opted into via `preserve_pinnable_kinds` [lib.rs:SessionRetentionPolicy::default].
- Retention decision: `should_preserve_event` returns `true` only when the event is pinned and `preserve_pinned` is set; the `_created_at` parameter is currently not consulted, and the `preserve_pinnable_kinds` field is not consulted by this function in the current implementation [lib.rs:SessionRetentionPolicy::should_preserve_event].

## Side Effects

- None

## Verification

- Run: `cargo test -p ck-memory`
