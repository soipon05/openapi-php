//! config_test.rs
//!
//! Tests for Config loading, defaults, and TOML parsing.

use openapi_php::config::{Config, Framework, PhpVersion};
use std::path::PathBuf;

// ─── Config::default() ────────────────────────────────────────────────────

#[test]
fn config_defaults_when_no_file() {
    let config = Config::default();
    assert_eq!(config.namespace, "App\\Generated");
    assert_eq!(config.output, None);
    assert_eq!(config.framework, Framework::Plain);
    assert_eq!(config.php_version, PhpVersion::Php81);
    assert_eq!(config.input, None);
}

// ─── Config::load() with no file present ──────────────────────────────────

#[test]
fn config_load_missing_file() {
    // A directory with no openapi-php.toml — should silently fall back to defaults.
    let dir = std::env::temp_dir();
    let config = Config::load(&dir).expect("load should not fail when no file found");
    assert_eq!(config.namespace, "App\\Generated");
    assert_eq!(config.output, None);
    assert_eq!(config.framework, Framework::Plain);
    assert_eq!(config.php_version, PhpVersion::Php81);
}

// ─── Config::from_toml_str() ──────────────────────────────────────────────

#[test]
fn config_toml_parse() {
    let toml = r#"
[generator]
namespace = "App\\Api"
output = "src/Generated"
framework = "laravel"
php_version = "8.2"

[input]
path = "openapi.yaml"
"#;
    let config = Config::from_toml_str(toml).expect("should parse successfully");
    assert_eq!(config.namespace, "App\\Api");
    assert_eq!(config.output, Some(PathBuf::from("src/Generated")));
    assert_eq!(config.framework, Framework::Laravel);
    assert_eq!(config.php_version, PhpVersion::Php82);
    assert_eq!(config.input, Some(PathBuf::from("openapi.yaml")));
}

#[test]
fn config_toml_parse_defaults_for_missing_fields() {
    let config = Config::from_toml_str("").expect("empty toml should use defaults");
    assert_eq!(config.namespace, "App\\Generated");
    assert_eq!(config.output, None);
    assert_eq!(config.framework, Framework::Plain);
    assert_eq!(config.php_version, PhpVersion::Php81);
    assert_eq!(config.input, None);
}

#[test]
fn config_toml_parse_symfony_php83() {
    let toml = r#"
[generator]
framework = "symfony"
php_version = "8.3"
"#;
    let config = Config::from_toml_str(toml).unwrap();
    assert_eq!(config.framework, Framework::Symfony);
    assert_eq!(config.php_version, PhpVersion::Php83);
}

#[test]
fn config_toml_unknown_framework_errors() {
    let toml = r#"
[generator]
framework = "rails"
"#;
    assert!(Config::from_toml_str(toml).is_err());
}

#[test]
fn config_toml_unknown_php_version_errors() {
    let toml = r#"
[generator]
php_version = "7.4"
"#;
    assert!(Config::from_toml_str(toml).is_err());
}

// ─── merge_cli: CLI values override config ────────────────────────────────

#[test]
fn merge_cli_overrides_namespace() {
    use openapi_php::config::CliOverrides;

    let config = Config::from_toml_str(
        r#"
[generator]
namespace = "From\\Config"
"#,
    )
    .unwrap();

    let merged = config.merge_cli(CliOverrides {
        namespace: Some("From\\Cli".to_string()),
        output: None,
        framework: None,
        php_version: None,
        input: None,
    });

    assert_eq!(merged.namespace, "From\\Cli");
}

#[test]
fn merge_cli_config_wins_when_cli_absent() {
    use openapi_php::config::CliOverrides;

    let config = Config::from_toml_str(
        r#"
[generator]
output = "config-out"

[input]
path = "config-spec.yaml"
"#,
    )
    .unwrap();

    let merged = config.merge_cli(CliOverrides {
        namespace: None,
        output: None,
        framework: None,
        php_version: None,
        input: None,
    });

    assert_eq!(merged.output, Some(PathBuf::from("config-out")));
    assert_eq!(merged.input, Some(PathBuf::from("config-spec.yaml")));
}
