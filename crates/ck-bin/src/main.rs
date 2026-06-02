use std::io::{self, BufRead};
use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;
use ck_adapter_filesystem::{FilesystemSourceAdapter, HeadingChunker, MarkdownNormalizer};
use ck_app::AppServices;
use ck_artifacts::FileArtifactStore;
use ck_config::{default_config, load_config};
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

    let service = McpService::with_app(status, app);

    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        println!("{}", ck_transport_stdio::handle_line(&service, &line)?);
    }

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
