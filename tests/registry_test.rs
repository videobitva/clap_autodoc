use clap::Parser;
use clap_autodoc::register;

#[derive(Clone, Debug, Parser)]
#[clap(rename_all = "kebab-case")]
#[register]
pub struct SimpleConfig {
    /// Test field
    #[clap(env = "TEST_FIELD", long)]
    pub test_field: String,
}

#[test]
fn test_registration_macro() {
    // This test just verifies that the registration macro compiles
    // and doesn't panic during compilation
    assert!(true);
}
