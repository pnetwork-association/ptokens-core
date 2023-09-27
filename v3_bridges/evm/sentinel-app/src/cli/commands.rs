use clap::Subcommand;

#[derive(Debug, Subcommand)]
#[command(rename_all = "camelCase")]
pub enum Commands {
    /// Test an endpoint
    TestEndpoint { endpoint: String },
}
