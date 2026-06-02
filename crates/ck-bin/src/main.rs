use std::io::{self, BufRead};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use ck_adapter_filesystem::{FilesystemSourceAdapter, HeadingChunker, MarkdownNormalizer};
use ck_app::AppServices;
use ck_artifacts::FileArtifactStore;
use ck_auth::BearerAuth;
use ck_config::{default_config, load_config};
use ck_core::ports::{AuthPort, StoragePort};
use ck_mcp::McpService;
use ck_ranker::LexicalRanker;
use ck_search_fts::FtsSearch;
use ck_storage_sqlite::SqliteStore;
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
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(io::stderr)
        .try_init()
        .ok();

    let cli = Cli::parse();
    match cli.command {
        Commands::Serve(args) => serve(args),
        Commands::Migrate(args) => migrate(args),
        Commands::Doctor(args) => doctor(args),
        Commands::ExportSchema => {
            println!("schema export is not implemented in Milestone 0");
            Ok(())
        }
    }
}

fn serve(args: RuntimeArgs) -> Result<()> {
    let (store, config) = open_store(&args)?;
    store.migrate()?;

    let data_dir = args.data_dir.unwrap_or_else(|| {
        let db_path = PathBuf::from(&config.storage.database_path);
        db_path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
    });

    let started_at = Utc::now().to_rfc3339();
    let storage_status = store.status()?;

    // Wire all adapters
    let app = wire_app(&store, &data_dir)?;

    let status = app.get_status(
        &started_at,
        storage_status,
        &config.retrieval.default_mode,
        config.retrieval.semantic_enabled,
    )?;

    let service = Arc::new(McpService::with_app(status, app));

    // Determine if SSE should be enabled
    let enable_sse = args.sse.is_some() || config.mcp.enable_sse;
    let sse_bind = args.sse.as_deref().unwrap_or(config.mcp.sse.bind.as_str());

    if enable_sse {
        // Wire auth
        let auth: Arc<dyn AuthPort> = if config.mcp.sse.require_auth {
            if config.auth.tokens.is_empty() {
                anyhow::bail!("SSE auth is required but no auth tokens are configured");
            }
            Arc::new(BearerAuth::new(config.auth.tokens.clone(), false))
        } else {
            Arc::new(BearerAuth::no_auth())
        };

        let sse_state = ck_transport_sse::SseState {
            mcp: service.clone(),
            auth,
        };

        // Start SSE server in a background thread
        let sse_bind_owned = sse_bind.to_string();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            rt.block_on(async {
                if let Err(e) = ck_transport_sse::serve_sse(sse_state, &sse_bind_owned).await {
                    tracing::error!(error = %e, "SSE server error");
                }
            });
        });

        tracing::info!(bind = sse_bind, "SSE MCP server started");
    }

    // Periodic refresh background thread
    let running = Arc::new(AtomicBool::new(true));
    if config.scheduler.enable_periodic_refresh {
        let refresh_store = store.clone();
        let refresh_app = wire_app(&refresh_store, &data_dir)?;
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
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
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

fn wire_app(store: &SqliteStore, data_dir: &str) -> Result<AppServices> {
    let db_path = store.path().to_path_buf();

    Ok(AppServices::new(
        Box::new(store.clone()),
        Box::new(store.clone()),
        Box::new(FileArtifactStore::new(PathBuf::from(data_dir))),
        Box::new(FtsSearch::open(db_path)?),
        Box::new(LexicalRanker),
        Box::new(FilesystemSourceAdapter),
        Box::new(MarkdownNormalizer),
        Box::new(HeadingChunker),
    ))
}
