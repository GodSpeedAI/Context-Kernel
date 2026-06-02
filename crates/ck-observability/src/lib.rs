use std::io;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

pub struct ObservabilityRuntimeConfig {
    pub log_format: String,
    pub otel_endpoint: Option<String>,
    pub service_name: String,
}

impl ObservabilityRuntimeConfig {
    pub fn otel_enabled(&self) -> bool {
        self.otel_endpoint.is_some()
    }
}

pub struct ObservabilityGuard {
    _inner: tracing::subscriber::DefaultGuard,
}

pub fn init_observability(config: ObservabilityRuntimeConfig) -> Result<ObservabilityGuard> {
    if config.otel_enabled() {
        tracing::warn!(
            endpoint = ?config.otel_endpoint,
            "OTEL endpoint configured but not yet supported; ignoring"
        );
    }

    let env_filter = EnvFilter::from_default_env();

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(io::stderr)
        .with_target(true);

    let subscriber: Box<dyn tracing::Subscriber + Send + Sync + 'static> =
        if config.log_format == "json" {
            Box::new(subscriber.json().finish())
        } else {
            Box::new(subscriber.finish())
        };

    let guard = tracing::subscriber::set_default(subscriber);

    Ok(ObservabilityGuard { _inner: guard })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_text_tracing_config_without_otel() {
        let config = ObservabilityRuntimeConfig {
            log_format: "text".to_string(),
            otel_endpoint: None,
            service_name: "test-service".to_string(),
        };

        assert_eq!(config.log_format, "text");
        assert!(config.otel_endpoint.is_none());
        assert!(!config.otel_enabled());
        assert_eq!(config.service_name, "test-service");
    }

    #[test]
    fn detects_otel_endpoint() {
        let config = ObservabilityRuntimeConfig {
            log_format: "json".to_string(),
            otel_endpoint: Some("http://localhost:4317".to_string()),
            service_name: "test-service".to_string(),
        };

        assert!(config.otel_enabled());
        assert_eq!(
            config.otel_endpoint.as_deref(),
            Some("http://localhost:4317")
        );
    }
}
