# ck-auth

## Overview

`ck-auth` is a minimal authentication adapter crate implementing `ck-core`'s `AuthPort` with bearer tokens, constant-time token comparison (via `subtle`), optional tool and corpus allowlists, and a local trust mode. The crate has a single source module and no dependencies beyond `ck-core` and `subtle`. It is wired into the `ck-bin` binary, which selects `BearerAuth::with_policy` or `BearerAuth::no_auth` during bootstrap.

## Subdirectories & Boundaries

- `src`: [`./src/README.md`](./src/README.md) - Component contract for `BearerAuth` (the `AuthPort` implementation).

## Verification

- Run: `cargo test -p ck-auth`
