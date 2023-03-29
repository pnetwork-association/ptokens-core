use clap::Args;

#[derive(Debug, Args)]
pub struct StartSentinelArgs {
    /// Disable the host syncer
    #[arg(long)]
    pub disable_host_syncer: bool,

    /// Disable the native syncer
    #[arg(long)]
    pub disable_native_syncer: bool,

    /// Disable the broadcaster
    #[arg(long)]
    pub disable_broadcaster: bool,
}
