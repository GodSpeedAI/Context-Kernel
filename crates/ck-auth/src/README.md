# ck-auth src

## Overview

Authentication adapter implementing the `ck-core` `AuthPort` for bearer-token authorization. `BearerAuth` validates tokens with constant-time comparison and enforces optional per-tool and per-corpus allowlists, with a local trust mode for unauthenticated operation. The crate's only dependencies are `ck-core` and `subtle`. It is composed into the `ck-bin` binary, which constructs `BearerAuth::with_policy` or `BearerAuth::no_auth` during bootstrap.

## Interface

- [lib.rs:BearerAuth] - `pub struct` implementing `AuthPort` (`validate_token`, `is_local_trust_mode`, `authorize_tool`, `authorize_corpus`); constructors `new(valid_tokens, local_trust_mode)`, `no_auth()`, and `with_policy(valid_tokens, local_trust_mode, allow_tools, allow_corpora)`.

## Invariants

- Token comparison uses constant-time equality via `subtle::ConstantTimeEq` [lib.rs:validate_token]
- A `Bearer ` prefix is optional and stripped before comparison [lib.rs:validate_token]
- With an empty token list, token validation succeeds only in local trust mode; otherwise it fails closed [lib.rs:token_is_authorized]
- Default allowlists are `["*"]` (permit all) for `new` and `no_auth`, and `with_policy` normalizes an empty allowlist to `["*"]` [lib.rs:normalize_allowlist]
- `authorize_tool` requires both a valid token and tool allowlist membership [lib.rs:authorize_tool]
- `authorize_corpus` requires both a valid token and corpus allowlist membership [lib.rs:authorize_corpus]
- Allowlist entries match either exactly or via the `*` wildcard [lib.rs:is_allowed]

## Side Effects

- None

## Verification

- Run: `cargo test -p ck-auth`
