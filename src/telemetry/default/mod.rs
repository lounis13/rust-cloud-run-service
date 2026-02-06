//! Default/local telemetry provider.
//!
//! This module provides a simple telemetry provider for local development
//! that optionally exports traces to a local OTLP collector.
//!
//! # Behavior
//!
//! - If `OTEL_EXPORTER_OTLP_ENDPOINT` is set: exports to that endpoint
//! - Otherwise: no-op (traces are not exported)
//!
//! # Example
//!
//! ```rust,ignore
//! use telemetry::default::DefaultProvider;
//! use telemetry::{TelemetryConfig, api::init_with_provider};
//!
//! let config = TelemetryConfig::from_env();
//! let provider = DefaultProvider;
//! init_with_provider(&provider, &config).await?;
//! ```

mod provider;

pub use provider::DefaultProvider;
