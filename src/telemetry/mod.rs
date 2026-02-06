//! Modular telemetry with pluggable providers.
//!
//! This module provides a flexible telemetry system built on OpenTelemetry
//! with support for multiple backends via feature flags.
//!
//! # Features
//!
//! - `telemetry-gcp`: Enable GCP Cloud Trace support
//!
//! # Quick Start
//!
//! ```rust,ignore
//! // Initialize from environment (uses Local backend by default)
//! telemetry::init().await?;
//! ```
//!
//! # Configuration
//!
//! ## Using the Builder
//!
//! ```rust,ignore
//! use telemetry::{TelemetryConfig, GcpConfig, LogFormat};
//!
//! let config = TelemetryConfig::builder()
//!     .service_name("my-service")
//!     .service_version("1.0.0")
//!     .log_level("debug")
//!     .gcp(GcpConfig::new("my-project"))
//!     .json()  // JSON logs for cloud
//!     .build();
//!
//! telemetry::init_with_config(&config).await?;
//! ```
//!
//! ## Log Formats
//!
//! - [`LogFormat::Pretty`]: Human-readable with colors (default for local dev)
//! - [`LogFormat::Json`]: Structured JSON (for cloud environments)
//!
//! ## Backends
//!
//! - [`TelemetryBackend::Local`]: Local development with optional OTLP export
//! - [`TelemetryBackend::Gcp`]: GCP Cloud Trace (requires `telemetry-gcp` feature)
//!
//! # Environment Variables
//!
//! | Variable | Description | Default |
//! |----------|-------------|---------|
//! | `OTEL_SERVICE_NAME` | Service name | `CARGO_PKG_NAME` |
//! | `OTEL_SERVICE_VERSION` | Service version | `CARGO_PKG_VERSION` |
//! | `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP endpoint | - |
//! | `RUST_LOG` | Log level filter | `info` |
//! | `LOG_FORMAT` | `pretty` or `json` | `pretty` |
//!
//! # Module Structure
//!
//! - [`api`]: Core trait and initialization functions
//! - [`config`]: Configuration types
//! - [`error`]: Error types
//! - [`default`]: Local/default provider
//! - [`gcp`]: GCP Cloud Trace provider (feature-gated)

#![allow(dead_code, unused_imports)] // Public API - not all items used internally

pub mod api;
pub mod config;
pub mod default;
pub mod error;
pub mod resource;
pub mod trace;

#[cfg(feature = "telemetry-gcp")]
pub mod gcp;
#[cfg(feature = "telemetry-gcp")]
pub use gcp::{GcpConfig, GcpPlatform};

// Re-exports
pub use api::{init, init_with_config, init_with_provider, TelemetryProvider};
pub use config::{LogFormat, TelemetryBackend, TelemetryConfig, TelemetryConfigBuilder};
pub use error::TelemetryError;



