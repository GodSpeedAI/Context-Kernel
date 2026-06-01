use std::io::{self, BufRead};

use anyhow::Result;
use chrono::Utc;
use ck_config::{default_config, load_config};
use ck_core::KernelStatus;
use ck_mcp::McpService;
use ck_storage_sqlite::SqliteStore;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "context-kernal")]
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
    let (store, config) = open_store(args)?;
    store.migrate()?;

    let status = KernelStatus::new(
        Utc::now().to_rfc3339(),
        store.status()?,
        config.retrieval.default_mode,
        config.retrieval.semantic_enabled,
    );
    let service = McpService::new(status);

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
    let (store, _) = open_store(args)?;
    store.migrate()?;
    println!("migrated {}", store.status()?.path);
    Ok(())
}

fn doctor(args: RuntimeArgs) -> Result<()> {
    let (store, _) = open_store(args)?;
    store.migrate()?;
    let status = store.status()?;
    println!(
        "storage={} path={} fts5_enabled={}",
        status.kind, status.path, status.fts5_enabled
    );
    Ok(())
}

fn open_store(args: RuntimeArgs) -> Result<(SqliteStore, ck_config::KernelConfig)> {
    let mut config = match args.config {
        Some(path) => load_config(path)?,
        None => default_config(),
    };

    if let Some(database) = args.database {
        config.storage.database_path = database;
    }

    let store = SqliteStore::open(&config.storage.database_path)?;
    Ok((store, config))
}
