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
}

#[derive(Clone, Debug, Parser)]
#[clap(rename_all = "kebab-case")]
#[generate(target = "tests/output/flattening_test_output.md", format = "flat")]
pub struct Config {
    /// Database configuration
    #[clap(flatten)]
    pub database: DatabaseConfig,

    /// Server port
    #[clap(env = "SERVER_PORT", long, default_value_t = 8080)]
    pub port: u16,
}

#[test]
fn test_flattening_expansion() {
    // Check if the file was created
    assert!(std::path::Path::new("tests/output/flattening_test_output.md").exists());

    // Read the generated content
    let content = std::fs::read_to_string("tests/output/flattening_test_output.md").unwrap();

    // Expected correct behavior - flattened fields should be expanded
    let expected = vec![
        "[//]: # (CONFIG_DOCS_START)",
        "",
        "| Field Name    | Type   | Required | Default | Details       | Group          |",
        "|---------------|--------|----------|---------|---------------|----------------|",
        "| postgres-host | String | Yes      | -       | Database host | DatabaseConfig |",
        "| postgres-port | u16    | No       | 5432    | Database port | DatabaseConfig |",
        "| port          | u16    | No       | 8080    | Server port   | Config         |",
        "",
        "[//]: # (CONFIG_DOCS_END)",
    ]
    .join("\n");

    // Verify that flattened fields are properly expanded
    assert_eq!(content.trim(), expected.trim());
}
