# SQLite Schema

Schema version 1 is installed by `ck-storage-sqlite`.

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

Operational notes:
- Run `context-kernal migrate --database <path>` before serving to create or update the database.
- Run `context-kernal export-schema` to print the SQL currently embedded in the binary.
- Retention GC deletes unpinned session events, inactive sessions, finished refresh jobs, disabled stale sources, and artifact metadata rows according to `[retention]` config.
- Retention GC does not delete artifact payload files yet; it removes SQLite metadata only.
