use clap::Parser;
use clap_autodoc::generate;

#[derive(Clone, Debug, Parser)]
#[clap(rename_all = "kebab-case")]
#[clap_autodoc::register]
pub struct DatabaseConfig {
    /// Database host
    #[clap(env = "POSTGRES_HOST", long)]
    pub postgres_host: String,

    /// Database port
    #[clap(env = "POSTGRES_PORT", long, default_value_t = 5432)]
    pub postgres_port: u16,

    #[clap(env = "POSTGRES_USER", long)]
    pub postgres_user: String,

    #[clap(env = "POSTGRES_PASSWORD", long)]
    pub postgres_password: String,

    #[clap(env = "POSTGRES_DATABASE", long, default_value = "data-ingestion")]
    pub postgres_database: String,
}

#[derive(Clone, Debug, Parser)]
#[clap(rename_all = "kebab-case")]
#[clap_autodoc::register]
pub struct RedisConfig {
    /// Redis host
    #[clap(env = "REDIS_HOST", long)]
    pub redis_host: String,

    /// Redis port
    #[clap(env = "REDIS_PORT", long, default_value_t = 6379)]
    pub redis_port: u16,
}

#[derive(Clone, Debug, Parser)]
#[clap(rename_all = "kebab-case")]
#[generate(target = "tests/output/nested_flat_output.md", format = "flat")]
pub struct NestedConfigFlat {
    /// Database configuration
    #[clap(flatten)]
    pub database: DatabaseConfig,

    /// Redis configuration  
    #[clap(flatten)]
    pub redis: RedisConfig,

    /// Server port
    #[clap(env = "SERVER_PORT", long, default_value_t = 8080)]
    pub port: u16,
}

#[derive(Clone, Debug, Parser)]
#[clap(rename_all = "kebab-case")]
#[generate(target = "tests/output/nested_grouped_output.md", format = "grouped")]
pub struct NestedConfigGrouped {
    /// Database configuration
    #[clap(flatten)]
    pub database: DatabaseConfig,

    /// Redis configuration  
    #[clap(flatten)]
    pub redis: RedisConfig,

    /// Server port
    #[clap(env = "SERVER_PORT", long, default_value_t = 8080)]
    pub port: u16,
}

#[test]
fn test_nested_flat_format() {
    // Check if the flat format file was created
    assert!(std::path::Path::new("tests/output/nested_flat_output.md").exists());

    // Read the generated content
    let content = std::fs::read_to_string("tests/output/nested_flat_output.md").unwrap();

    // Expected output for flat format with expanded flattened fields
    let expected = vec![
        "[//]: # (CONFIG_DOCS_START)",
        "",
        "| Field Name        | Type   | Required | Default        | Details       | Group            |",
        "|-------------------|--------|----------|----------------|---------------|------------------|",
        "| postgres-host     | String | Yes      | -              | Database host | DatabaseConfig   |",
        "| postgres-port     | u16    | No       | 5432           | Database port | DatabaseConfig   |",
        "| postgres-user     | String | Yes      | -              |               | DatabaseConfig   |",
        "| postgres-password | String | Yes      | -              |               | DatabaseConfig   |",
        "| postgres-database | String | No       | data-ingestion |               | DatabaseConfig   |",
        "| redis-host        | String | Yes      | -              | Redis host    | RedisConfig      |",
        "| redis-port        | u16    | No       | 6379           | Redis port    | RedisConfig      |",
        "| port              | u16    | No       | 8080           | Server port   | NestedConfigFlat |",
        "",
        "[//]: # (CONFIG_DOCS_END)"
    ].join("\n");

    // Compare the generated content with expected output
    assert_eq!(content.trim(), expected.trim());
}

#[test]
fn test_nested_grouped_format() {
    assert!(std::path::Path::new("tests/output/nested_grouped_output.md").exists());

    let content = std::fs::read_to_string("tests/output/nested_grouped_output.md").unwrap();

    let expected = vec![
        "[//]: # (CONFIG_DOCS_START)",
        "",
        "## DatabaseConfig Configuration",
        "",
        "| Field Name        | Type   | Required | Default        | Details       |",
        "|-------------------|--------|----------|----------------|---------------|",
        "| postgres-host     | String | Yes      | -              | Database host |",
        "| postgres-port     | u16    | No       | 5432           | Database port |",
        "| postgres-user     | String | Yes      | -              |               |",
        "| postgres-password | String | Yes      | -              |               |",
        "| postgres-database | String | No       | data-ingestion |               |",
        "",
        "## RedisConfig Configuration",
        "",
        "| Field Name | Type   | Required | Default | Details    |",
        "|------------|--------|----------|---------|------------|",
        "| redis-host | String | Yes      | -       | Redis host |",
        "| redis-port | u16    | No       | 6379    | Redis port |",
        "",
        "## NestedConfigGrouped Configuration",
        "",
        "| Field Name | Type | Required | Default | Details     |",
        "|------------|------|----------|---------|-------------|",
        "| port       | u16  | No       | 8080    | Server port |",
        "",
        "",
        "",
        "[//]: # (CONFIG_DOCS_END)",
    ]
    .join("\n");

    assert_eq!(content.trim(), expected.trim());
}
