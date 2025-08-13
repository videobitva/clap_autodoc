use clap::Parser;
use clap_autodoc::generate;

#[derive(Clone, Debug, Parser)]
#[clap(rename_all = "kebab-case", rename_all_env = "SCREAMING_SNAKE_CASE")]
#[generate(target = "tests/output/test_output.md")]
pub struct TestConfig {
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

    #[clap(env = "POSTGRES_CONNECTION_POOL", long, default_value_t = 5)]
    pub postgres_connection_pool: u32,
}

#[test]
fn test_basic_functionality() {
    // The macro should generate the documentation when the struct is defined
    // We can verify this by checking if the test_output.md file was created
    assert!(std::path::Path::new("tests/output/test_output.md").exists());

    // Read the generated content
    let content = std::fs::read_to_string("tests/output/test_output.md").unwrap();

    // Expected output for flat table format (using tabled's formatting)
    let expected = vec![
        "[//]: # (CONFIG_DOCS_START)",
        "",
        "| Field Name               | Type   | Required | Default        | Details       | Group      |",
        "|--------------------------|--------|----------|----------------|---------------|------------|",
        "| postgres-host            | String | Yes      | -              | Database host | TestConfig |",
        "| postgres-port            | u16    | No       | 5432           | Database port | TestConfig |",
        "| postgres-user            | String | Yes      | -              |               | TestConfig |",
        "| postgres-password        | String | Yes      | -              |               | TestConfig |",
        "| postgres-database        | String | No       | data-ingestion |               | TestConfig |",
        "| postgres-connection-pool | u32    | No       | 5              |               | TestConfig |",
        "",
        "[//]: # (CONFIG_DOCS_END)"
    ].join("\n");

    // Compare the generated content with expected output
    assert_eq!(content.trim(), expected.trim());

    println!("Generated markdown content:\n{}", content);
}
