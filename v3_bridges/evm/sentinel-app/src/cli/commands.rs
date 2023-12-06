use clap::Subcommand;

#[derive(Debug, Subcommand)]
#[command(rename_all = "camelCase")]
pub enum Commands {
    /// Test an endpoint
    TestEndpoint { endpoint: String },

    /// Get submission materail for given block number from given endpoint
    GetSubMat { block_num: u64, endpoint: String },
}
