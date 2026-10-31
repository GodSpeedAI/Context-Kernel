# ck-scheduler Source

## Overview
This crate implements refresh-queue orchestration for Context Kernel as an application-layer scheduler over the `SchedulerPort` contract. `RefreshScheduler` wraps an injected `Box<dyn StoragePort>` (defined in `ck-core`) and provides the refresh job lifecycle: enqueue with coalescing, claim, complete/fail with source-health updates, stale-source listing, and per-status job counting. It performs no persistence of its own - all job and source mutations are delegated through the storage port, so the crate contains no SQL. It is a workspace member but is not composed into the `ck-bin` binary: no crate outside `ck-scheduler` references `RefreshScheduler` or its `SchedulerPort` implementation (verified against the workspace tree; only the workspace manifest and docs mention the package).

## Interface
- [lib.rs:RefreshScheduler] - Public struct holding the injected `Box<dyn StoragePort>`; the sole `SchedulerPort` implementation in the crate.
- [lib.rs:RefreshScheduler::new] - Constructs the scheduler from a storage port.
- [lib.rs:RefreshScheduler::enqueue_refresh] - Creates a queued `RefreshJob` for a source and delegates insertion to `storage.insert_active_refresh_job`, which coalesces onto an existing active job when present.
- [lib.rs:RefreshScheduler::claim_next_job] - Iterates all sources from `storage.list_sources()` and returns the first job successfully claimed via `storage.claim_next_refresh_job_for_source`, or `None`.
- [lib.rs:RefreshScheduler::complete_job] - Transitions a job to `Completed`, records `finished_at` and `stats_json`, and sets the source's health to `Healthy`.
- [lib.rs:RefreshScheduler::fail_job] - Transitions a job to `Failed`, records `finished_at` and the error, and sets the source's health to `Degraded`.
- [lib.rs:RefreshScheduler::list_stale_sources] - Delegates to `storage.list_stale_sources()` and returns source ids only.
- [lib.rs:RefreshScheduler::count_jobs_by_status] - Maps `JobStatus` to its lowercase string form and delegates counting to storage.

## Invariants
- Trigger mapping: [lib.rs:RefreshScheduler::enqueue_refresh] maps the trigger strings `"startup"`, `"scheduled"`, `"webhook"`, and `"dependency_change"` to the corresponding `JobTrigger` variants; any other string becomes `JobTrigger::Manual`.
- Job identity and initial state: new jobs get `job_{ULID}` ids, `JobStatus::Queued`, empty `stats_json` (`"{}"`), and RFC3339 UTC timestamps from `chrono::Utc::now` ([lib.rs:RefreshScheduler::enqueue_refresh]).
- Coalescing rule: enqueue relies on `StoragePort::insert_active_refresh_job` to return the already-active (queued/running) job for the source instead of inserting a duplicate; the scheduler logs `coalescing: active job already exists` when the returned job id differs from the one it built ([lib.rs:RefreshScheduler::enqueue_refresh]).
- Claim semantics: at most one job is claimed per `claim_next_job` call, in storage's source ordering; the queued-to-running transition (including `started_at`) is performed by the storage port's `claim_next_refresh_job_for_source` ([lib.rs:RefreshScheduler::claim_next_job]).
- Health coupling: completing a job always sets the source health to `Healthy` with no error; failing a job always sets it to `Degraded` carrying the error string ([lib.rs:RefreshScheduler::complete_job], [lib.rs:RefreshScheduler::fail_job]).
- Unknown job ids are a no-op: `complete_job` and `fail_job` return `Ok(())` without writing when `get_refresh_job` yields `None` ([lib.rs:RefreshScheduler::complete_job], [lib.rs:RefreshScheduler::fail_job]).
- Status vocabulary: [lib.rs:RefreshScheduler::count_jobs_by_status] maps `JobStatus` variants to exactly `"queued"`, `"running"`, `"completed"`, `"failed"`, `"cancelled"` when querying storage.

## Side Effects
- Persistence via the injected `StoragePort`: job insert/upsert/claim, source health updates, and source/job listing and counting are all delegated to the storage implementation ([lib.rs:RefreshScheduler]).
- Emits `tracing::debug` events and `#[instrument]` spans carrying `source_id` / `job_id` fields ([lib.rs:RefreshScheduler::enqueue_refresh] and siblings).
- Reads the system clock for RFC3339 timestamps via `chrono::Utc::now` ([lib.rs:RefreshScheduler::enqueue_refresh], [lib.rs:RefreshScheduler::complete_job], [lib.rs:RefreshScheduler::fail_job]).
- No direct filesystem, network, or database access, and no global state.

## Verification
- Run: `cargo test -p ck-scheduler`
