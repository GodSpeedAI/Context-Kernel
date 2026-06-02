use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelStatus {
    pub service: ServiceStatus,
    pub storage: StorageStatus,
    pub sources: SourcesStatus,
    pub sessions: SessionsStatus,
    pub jobs: JobsStatus,
    pub retrieval: RetrievalStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub version: String,
    pub started_at: String,
    pub healthy: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageStatus {
    pub kind: String,
    pub path: String,
    pub fts5_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcesStatus {
    pub total: u64,
    pub enabled: u64,
    pub degraded: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionsStatus {
    pub active: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobsStatus {
    pub queued: u64,
    pub running: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetrievalStatus {
    pub default_mode: String,
    pub semantic_enabled: bool,
}

impl KernelStatus {
    pub fn new(
        started_at: String,
        storage: StorageStatus,
        default_mode: impl Into<String>,
        semantic_enabled: bool,
    ) -> Self {
        Self {
            service: ServiceStatus {
                name: "context-kernel".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                started_at,
                healthy: true,
            },
            storage,
            sources: SourcesStatus {
                total: 0,
                enabled: 0,
                degraded: 0,
            },
            sessions: SessionsStatus { active: 0 },
            jobs: JobsStatus {
                queued: 0,
                running: 0,
            },
            retrieval: RetrievalStatus {
                default_mode: default_mode.into(),
                semantic_enabled,
            },
        }
    }

    pub fn placeholder() -> Self {
        Self {
            service: ServiceStatus {
                name: "context-kernel".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                started_at: "not-started".to_string(),
                healthy: true,
            },
            storage: StorageStatus {
                kind: "sqlite".to_string(),
                path: "kernel.db".to_string(),
                fts5_enabled: true,
            },
            sources: SourcesStatus {
                total: 0,
                enabled: 0,
                degraded: 0,
            },
            sessions: SessionsStatus { active: 0 },
            jobs: JobsStatus {
                queued: 0,
                running: 0,
            },
            retrieval: RetrievalStatus {
                default_mode: "hybrid".to_string(),
                semantic_enabled: false,
            },
        }
    }
}
