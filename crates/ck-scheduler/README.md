# ck-scheduler

## Overview
`ck-scheduler` (package `ck-scheduler`) implements refresh-queue orchestration for Context Kernel: enqueueing refresh jobs with coalescing per source, claiming queued jobs, completing or failing jobs with coupled source-health updates, and listing stale sources and job counts. `RefreshScheduler` implements the `SchedulerPort` from `ck-core` entirely over an injected `StoragePort`, keeping the crate free of persistence details. The crate is a workspace member but is currently not composed into the `ck-bin` binary - no other crate in the workspace references `RefreshScheduler` or `SchedulerPort`.

## Subdirectories & Boundaries
- `src`: [./src/README.md](./src/README.md) - Component contract for `RefreshScheduler`: trigger-string mapping, job id and lifecycle state rules, enqueue coalescing, claim semantics, and health-coupled complete/fail transitions.

## Verification
- Run: `cargo test -p ck-scheduler`
