# SQLite Schema

Milestone 0 installs schema version 1 through `ck-storage-sqlite`.

Implemented tables:
- `schema_migrations`
- `sources`
- `corpora`
- `corpus_sources`
- `library_identities`
- `library_aliases`
- `documents`
- `chunks`
- `sessions`
- `session_events`
- `artifacts`
- `refresh_jobs`

Implemented FTS5 tables:
- `chunks_fts`
- `session_events_fts`
- `library_aliases_fts`
