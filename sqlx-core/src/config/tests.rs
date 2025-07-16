use crate::config::{self, Config};
use std::collections::BTreeSet;
use std::sync::Once;

// Initialize environment variables once for all tests
static INIT: Once = Once::new();

fn init_test_env() {
    INIT.call_once(|| {
        // Set consistent test values that won't interfere with each other
        std::env::set_var("SQLX_MIGRATIONS_TABLE", "test_migrations");
        std::env::set_var("SQLX_MIGRATIONS_SCHEMA", "test_schema");
    });
}

#[test]
fn reference_parses_as_config() {
    let config: Config = toml::from_str(include_str!("reference.toml"))
        // The `Display` impl of `toml::Error` is *actually* more useful than `Debug`
        .unwrap_or_else(|e| panic!("expected reference.toml to parse as Config: {e}"));

    assert_common_config(&config.common);
    assert_macros_config(&config.macros);
    assert_migrate_config(&config.migrate);
}

fn assert_common_config(config: &config::common::Config) {
    assert_eq!(config.database_url_var.as_deref(), Some("FOO_DATABASE_URL"));
    assert_eq!(config.drivers.sqlite.load_extensions[1].as_str(), "vsv");
}

fn assert_macros_config(config: &config::macros::Config) {
    use config::macros::*;

    assert_eq!(config.preferred_crates.date_time, DateTimeCrate::Chrono);
    assert_eq!(config.preferred_crates.numeric, NumericCrate::RustDecimal);

    // Type overrides
    // Don't need to cover everything, just some important canaries.
    assert_eq!(config.type_override("UUID"), Some("crate::types::MyUuid"));

    assert_eq!(config.type_override("foo"), Some("crate::types::Foo"));

    assert_eq!(config.type_override(r#""Bar""#), Some("crate::types::Bar"),);

    assert_eq!(
        config.type_override(r#""Foo".bar"#),
        Some("crate::schema::foo::Bar"),
    );

    assert_eq!(
        config.type_override(r#""Foo"."Bar""#),
        Some("crate::schema::foo::Bar"),
    );

    // Column overrides
    assert_eq!(
        config.column_override("foo", "bar"),
        Some("crate::types::Bar"),
    );

    assert_eq!(
        config.column_override("foo", r#""Bar""#),
        Some("crate::types::Bar"),
    );

    assert_eq!(
        config.column_override(r#""Foo""#, "bar"),
        Some("crate::types::Bar"),
    );

    assert_eq!(
        config.column_override(r#""Foo""#, r#""Bar""#),
        Some("crate::types::Bar"),
    );

    assert_eq!(
        config.column_override("my_schema.my_table", "my_column"),
        Some("crate::types::MyType"),
    );

    assert_eq!(
        config.column_override(r#""My Schema"."My Table""#, r#""My Column""#),
        Some("crate::types::MyType"),
    );
}

fn assert_migrate_config(config: &config::migrate::Config) {
    use config::migrate::*;

    assert_eq!(config.table_name.as_deref(), Some("foo._sqlx_migrations"));
    assert_eq!(config.migrations_dir.as_deref(), Some("foo/migrations"));

    let ignored_chars = BTreeSet::from([' ', '\t', '\r', '\n', '\u{FEFF}']);

    assert_eq!(config.ignored_chars, ignored_chars);

    assert_eq!(
        config.defaults.migration_type,
        DefaultMigrationType::Reversible
    );
    assert_eq!(
        config.defaults.migration_versioning,
        DefaultVersioning::Sequential
    );

    // Test PostgreSQL schema configuration
    assert_eq!(
        config.drivers.postgres.schema.as_deref(),
        Some("my_migrations")
    );
}

#[test]
fn test_migrate_env_var_support() {
    use config::migrate::Config;

    init_test_env();

    // Test that environment variables are properly read
    let config = Config::default();

    assert_eq!(config.table_name(), "test_migrations");
    assert_eq!(config.postgres_schema(), Some("test_schema".to_string()));
}

#[test]
fn test_migrate_defaults_without_env() {
    use config::migrate::Config;

    // This test verifies behavior when env vars are NOT set
    // We test this by creating the config with explicit None values
    let config = Config {
        create_schemas: Default::default(),
        table_name: None,
        migrations_dir: Default::default(),
        ignored_chars: Default::default(),
        defaults: Default::default(),
        drivers: config::migrate::Drivers {
            postgres: config::migrate::Postgres { schema: None },
        },
    };

    assert_eq!(config.table_name(), "_sqlx_migrations");
    assert_eq!(config.postgres_schema(), None);
}
