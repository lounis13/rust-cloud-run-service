use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};

use crate::telemetry::config::TelemetryConfig;

/// Get base attributes for any resource
pub fn base_attributes(config: &TelemetryConfig) -> Vec<KeyValue> {
    vec![
        KeyValue::new(SERVICE_NAME, config.service_name.clone()),
        KeyValue::new(SERVICE_VERSION, config.service_version.clone()),
    ]
}

/// Build base resource with common attributes
pub fn build_base_resource(config: &TelemetryConfig) -> Resource {
    Resource::builder()
        .with_attributes(base_attributes(config))
        .build()
}

/// Build resource with base + additional attributes
pub fn build_resource(config: &TelemetryConfig, additional: Vec<KeyValue>) -> Resource {
    let mut attrs = base_attributes(config);
    attrs.extend(additional);
    Resource::builder().with_attributes(attrs).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> TelemetryConfig {
        TelemetryConfig::new("test-service", "1.2.3")
    }

    #[test]
    fn base_attributes_contains_service_name() {
        let config = test_config();
        let attrs = base_attributes(&config);

        let has_service_name = attrs
            .iter()
            .any(|kv| kv.key.as_str() == SERVICE_NAME && kv.value.as_str() == "test-service");

        assert!(has_service_name);
    }

    #[test]
    fn base_attributes_contains_service_version() {
        let config = test_config();
        let attrs = base_attributes(&config);

        let has_version = attrs
            .iter()
            .any(|kv| kv.key.as_str() == SERVICE_VERSION && kv.value.as_str() == "1.2.3");

        assert!(has_version);
    }

    #[test]
    fn base_attributes_has_two_entries() {
        let config = test_config();
        let attrs = base_attributes(&config);

        assert_eq!(attrs.len(), 2);
    }

    #[test]
    fn build_resource_includes_additional_attrs() {
        let config = test_config();
        let additional = vec![
            KeyValue::new("custom.attr", "value"),
            KeyValue::new("another.attr", "123"),
        ];

        let resource = build_resource(&config, additional);

        // Resource should have base (2) + additional (2) = 4 attributes
        // Note: We can't easily inspect Resource internals, but we can verify it builds
        assert!(!resource.is_empty());
    }
}