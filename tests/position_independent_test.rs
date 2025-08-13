use clap::Parser;
use clap_autodoc::{generate, register};

// Nested struct defined BEFORE main struct
#[derive(Clone, Debug, Parser)]
#[clap(rename_all = "kebab-case")]
#[register]
pub struct DatabaseConfig {
    /// Database host
    #[clap(env = "DB_HOST", long)]
    pub db_host: String,

    /// Database port
    #[clap(env = "DB_PORT", long, default_value_t = 5432)]
    pub db_port: u16,
}

// Main config struct
#[derive(Clone, Debug, Parser)]
#[clap(rename_all = "kebab-case")]
#[generate(
    target = "tests/output/position_independent_output.md",
    format = "flat"
)]
pub struct MainConfig {
    /// Server port
    #[clap(env = "SERVER_PORT", long, default_value_t = 8080)]
    pub port: u16,

    /// Database configuration
    #[clap(flatten)]
    pub database: DatabaseConfig,

    /// Cache configuration  
    #[clap(flatten)]
    pub cache: CacheConfig,
}

// Nested struct defined AFTER main struct
#[derive(Clone, Debug, Parser)]
#[clap(rename_all = "kebab-case")]
#[register]
pub struct CacheConfig {
    /// Cache host
    #[clap(env = "CACHE_HOST", long)]
    pub cache_host: String,

    /// Cache TTL in seconds
    #[clap(env = "CACHE_TTL", long, default_value_t = 3600)]
    pub cache_ttl: u32,
}

#[test]
fn test_position_independence() {
    assert!(std::path::Path::new("tests/output/position_independent_output.md").exists());

    let content = std::fs::read_to_string("tests/output/position_independent_output.md").unwrap();

    let expected = vec![
        "[//]: # (CONFIG_DOCS_START)",
        "",
        "| Field Name | Type   | Required | Default | Details              | Group          |",
        "|------------|--------|----------|---------|----------------------|----------------|",
        "| port       | u16    | No       | 8080    | Server port          | MainConfig     |",
        "| db-host    | String | Yes      | -       | Database host        | DatabaseConfig |",
        "| db-port    | u16    | No       | 5432    | Database port        | DatabaseConfig |",
        "| cache-host | String | Yes      | -       | Cache host           | CacheConfig    |",
        "| cache-ttl  | u32    | No       | 3600    | Cache TTL in seconds | CacheConfig    |",
        "",
        "[//]: # (CONFIG_DOCS_END)",
    ]
    .join("\n");

    assert_eq!(content.trim(), expected.trim());
}
