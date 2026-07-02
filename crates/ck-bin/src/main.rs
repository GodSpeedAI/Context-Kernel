use std::io::{self, BufRead};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use ck_adapter_filesystem::{FilesystemSourceAdapter, HeadingChunker, MarkdownNormalizer};
use ck_app::{AppServices, RetentionPolicy};
use ck_artifacts::FileArtifactStore;
use ck_auth::BearerAuth;
use ck_config::{default_config, load_config};
use ck_core::ports::{AuthPort, EmbeddingPort, StoragePort, VectorSearchPort};
use ck_embeddings::{NoopEmbeddingProvider, OpenAIEmbeddingProvider};
use ck_mcp::{AclContextAgent, McpService};
use ck_ranker::LexicalRanker;
use ck_search_fts::FtsSearch;
use ck_storage_sqlite::SqliteStore;
use ck_vector::SqliteVectorStore;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "context-kernel")]
#[command(about = "Context Kernel MCP service")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Serve(RuntimeArgs),
    Migrate(RuntimeArgs),
    Doctor(RuntimeArgs),
    ExportSchema,
}

#[derive(Debug, Parser)]
struct RuntimeArgs {
    #[arg(long)]
    config: Option<String>,
    #[arg(long)]
    database: Option<String>,
    #[arg(long)]
    data_dir: Option<String>,
    #[arg(long)]
    sse: Option<String>,
}

fn main() -> Result<()> {
    ck_observability::init_observability(ck_observability::ObservabilityRuntimeConfig {
        log_format: "text".to_string(),
        otel_endpoint: None,
        service_name: "context-kernal".to_string(),
    })
    .ok();

    let cli = Cli::parse();
    match cli.command {
        Commands::Serve(args) => serve(args),
        Commands::Migrate(args) => migrate(args),
        Commands::Doctor(args) => doctor(args),
        Commands::ExportSchema => {
            println!("{}", SqliteStore::schema_sql());
            Ok(())
        }
    }
}

fn serve(args: RuntimeArgs) -> Result<()> {
    let (store, config) = open_store(&args)?;
    store.migrate()?;

    let data_dir = args.data_dir.clone().unwrap_or_else(|| {
        let db_path = PathBuf::from(&config.storage.database_path);
        db_path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
    });

    let started_at = Utc::now().to_rfc3339();
    let storage_status = store.status()?;

    // Wire all adapters
    let app = wire_app(&store, &data_dir, &config)?;

    if config.retention.enabled {
        let report = app.run_retention_gc(RetentionPolicy {
            session_event_days: config.retention.session_event_days,
            inactive_session_days: config.retention.inactive_session_days,
            refresh_job_days: config.retention.refresh_job_days,
            artifact_days: config.retention.artifact_days,
            disabled_source_days: config.retention.disabled_source_days,
        })?;
        tracing::info!(
            session_events_deleted = report.session_events_deleted,
            inactive_sessions_deleted = report.inactive_sessions_deleted,
            refresh_jobs_deleted = report.refresh_jobs_deleted,
            artifacts_deleted = report.artifacts_deleted,
            disabled_sources_deleted = report.disabled_sources_deleted,
            "startup retention gc applied"
        );
    }

    let semantic_enabled = config.embeddings.enabled && app.semantic_available();

    let status = app.get_status(
        &started_at,
        storage_status,
        &config.retrieval.default_mode,
        semantic_enabled,
    )?;

    let mcp_auth: Arc<dyn AuthPort> = if config.mcp.sse.require_auth {
        if config.auth.tokens.is_empty() {
            anyhow::bail!("SSE auth is required but no auth tokens are configured");
        }
        Arc::new(BearerAuth::with_policy(
            config.auth.tokens.clone(),
            false,
            config.authorization.allow_tools.clone(),
            config.authorization.allow_corpora.clone(),
        ))
    } else {
        Arc::new(BearerAuth::no_auth())
    };

    // WP-5 (F-05): attach the ACL context agent so the loop-facing
    // `context_required` tool is served. The corpus root + learning-proposals
    // ledger paths are config/env-driven (absent → the tool reports itself as
    // unconfigured, rather than disabling the whole server).
    let context_agent = build_context_agent();
    let mut service = Arc::new(match context_agent {
        Some(agent) => McpService::with_app_and_auth(status, app, mcp_auth.clone())
            .with_context_agent(agent),
        None => McpService::with_app_and_auth(status, app, mcp_auth.clone()),
    });

    // Determine if SSE should be enabled
    let enable_sse = args.sse.is_some() || config.mcp.enable_sse;
    let sse_bind = args.sse.as_deref().unwrap_or(config.mcp.sse.bind.as_str());

    let sse_reload_state: Option<std::sync::Arc<ck_transport_sse::ReloadableSseState>> =
        if enable_sse {
            let state = std::sync::Arc::new(ck_transport_sse::ReloadableSseState::new(
                service.clone(),
                mcp_auth.clone(),
            ));
            let state_clone = state.clone();
            let sse_bind_owned = sse_bind.to_string();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async {
                    if let Err(e) = ck_transport_sse::serve_sse(state_clone, &sse_bind_owned).await
                    {
                        tracing::error!(error = %e, "SSE server error");
                    }
                });
            });
            tracing::info!(bind = sse_bind, "SSE MCP server started");
            Some(state)
        } else {
            None
        };

    // Periodic refresh background thread
    let running = Arc::new(AtomicBool::new(true));
    if config.scheduler.enable_periodic_refresh {
        let refresh_store = store.clone();
        let refresh_config = config.clone();
        let refresh_app = wire_app(&refresh_store, &data_dir, &refresh_config)?;
        let interval_secs = config.scheduler.refresh_interval_secs;
        let refresh_running = running.clone();

        std::thread::spawn(move || {
            tracing::info!(interval_secs, "periodic refresh thread started");
            while refresh_running.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_secs(interval_secs));

                if !refresh_running.load(Ordering::Relaxed) {
                    break;
                }

                match refresh_store.list_stale_sources() {
                    Ok(stale) => {
                        if stale.is_empty() {
                            tracing::debug!("no stale sources to refresh");
                        } else {
                            tracing::info!(count = stale.len(), "refreshing stale sources");
                            for source in &stale {
                                let req = ck_app::RefreshSourceRequest {
                                    source_id: source.source_id.clone(),
                                };
                                match refresh_app.refresh_source(req) {
                                    Ok(resp) => {
                                        tracing::info!(
                                            source_id = %source.source_id,
                                            job_id = %resp.job.job_id,
                                            "refreshed stale source"
                                        );
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            source_id = %source.source_id,
                                            error = %e,
                                            "failed to refresh stale source"
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "failed to list stale sources"
                        );
                    }
                }
            }
            tracing::info!("periodic refresh thread stopped");
        });
    }

    // Always serve stdio (stdin loop)
    let mut active_config = config.clone();
    let config_path = args.config.clone();
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        if active_config.reload.enabled {
            if let Some(path) = &config_path {
                let mut reloaded = load_config(path)?;
                if let Some(database) = &args.database {
                    reloaded.storage.database_path = database.clone();
                }
                if reloaded != active_config {
                    tracing::info!(config_path = path, "configuration reloaded");
                    let reload_store = SqliteStore::open(&reloaded.storage.database_path)?;
                    reload_store.migrate()?;
                    let reload_storage_status = reload_store.status()?;
                    let reload_app = wire_app(&reload_store, &data_dir, &reloaded)?;
                    let reload_semantic_enabled =
                        reloaded.embeddings.enabled && reload_app.semantic_available();
                    let reload_status = reload_app.get_status(
                        &started_at,
                        reload_storage_status,
                        &reloaded.retrieval.default_mode,
                        reload_semantic_enabled,
                    )?;
                    service = Arc::new(McpService::with_app(reload_status, reload_app));
                    active_config = reloaded;
                    if let Some(sse_state) = &sse_reload_state {
                        let reload_mcp_auth: Arc<dyn AuthPort> =
                            if active_config.mcp.sse.require_auth {
                                Arc::new(BearerAuth::with_policy(
                                    active_config.auth.tokens.clone(),
                                    false,
                                    active_config.authorization.allow_tools.clone(),
                                    active_config.authorization.allow_corpora.clone(),
                                ))
                            } else {
                                Arc::new(BearerAuth::no_auth())
                            };
                        sse_state.replace_service(service.clone());
                        sse_state.replace_auth(reload_mcp_auth);
                        tracing::info!("SSE state reloaded with new config");
                    }
                }
            }
        }

        println!("{}", ck_transport_stdio::handle_line(&service, &line)?);
    }

    running.store(false, Ordering::Relaxed);

    Ok(())
}

fn migrate(args: RuntimeArgs) -> Result<()> {
    let (store, _) = open_store(&args)?;
    store.migrate()?;
    println!("migrated {}", store.status()?.path);
    Ok(())
}

fn doctor(args: RuntimeArgs) -> Result<()> {
    let (store, _) = open_store(&args)?;
    store.migrate()?;
    let status = store.status()?;
    println!(
        "storage={} path={} fts5_enabled={}",
        status.kind, status.path, status.fts5_enabled
    );
    Ok(())
}

fn open_store(args: &RuntimeArgs) -> Result<(SqliteStore, ck_config::KernelConfig)> {
    let mut config = match &args.config {
        Some(path) => load_config(path)?,
        None => default_config(),
    };

    if let Some(database) = &args.database {
        config.storage.database_path = database.clone();
    }

    let store = SqliteStore::open(&config.storage.database_path)?;
    Ok((store, config))
}

fn wire_app(
    store: &SqliteStore,
    data_dir: &str,
    config: &ck_config::KernelConfig,
) -> Result<AppServices> {
    let db_path = store.path().to_path_buf();

    let mut app = AppServices::new(
        Box::new(store.clone()),
        Box::new(store.clone()),
        Box::new(FileArtifactStore::new(PathBuf::from(data_dir))),
        Box::new(FtsSearch::open(db_path.clone())?),
        Box::new(LexicalRanker),
        Box::new(FilesystemSourceAdapter),
        Box::new(MarkdownNormalizer),
        Box::new(HeadingChunker),
    );

    // Wire semantic retrieval when configured
    if config.embeddings.enabled && config.embeddings.provider != "none" {
        let embedding: Box<dyn EmbeddingPort> = if config.embeddings.provider == "openai" {
            Box::new(OpenAIEmbeddingProvider::new(
                config.embeddings.api_base.clone(),
                config.embeddings.api_key.clone(),
                config.embeddings.model.clone(),
                config.embeddings.dimensions,
            ))
        } else {
            tracing::warn!(
                provider = %config.embeddings.provider,
                "unknown embedding provider, falling back to noop"
            );
            Box::new(NoopEmbeddingProvider)
        };

        let vector_store: Box<dyn VectorSearchPort> = Box::new(SqliteVectorStore::open(&db_path)?);

        app = app.with_semantic(embedding, vector_store);

        tracing::info!(
            provider = %config.embeddings.provider,
            model = %config.embeddings.model,
            dimensions = config.embeddings.dimensions,
            "semantic retrieval configured"
        );
    }

    Ok(app)
}

/// WP-5 (F-05): build the ACL context agent from env-configured paths.
/// `CK_CONTEXT_CORPUS_ROOT` enables filesystem-corpus citation; the optional
/// `CK_LEARNING_PROPOSALS_LEDGER` adds the GodSpeed-Agent learning-proposals
/// corpus. Returns None when neither is set (the `context_required` tool then
/// reports itself as unconfigured).
fn build_context_agent() -> Option<AclContextAgent> {
    let corpus_root = std::env::var("CK_CONTEXT_CORPUS_ROOT").ok().filter(|s| !s.is_empty());
    let learning_ledger = std::env::var("CK_LEARNING_PROPOSALS_LEDGER")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    let root = corpus_root.map(PathBuf::from);
    if root.is_none() && learning_ledger.is_none() {
        return None;
    }
    Some(AclContextAgent {
        corpus_root: root.unwrap_or_default(),
        learning_proposals_ledger: learning_ledger,
    })
}
