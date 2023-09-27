mod commands;
mod handle_commands;
use common_sentinel::SentinelError;
use serde_json::json;

pub(crate) use self::commands::Commands;
use self::handle_commands::handle_test_endpoint;

pub async fn handle_cli(cmds: Commands) -> Result<String, SentinelError> {
    let result = match cmds {
        Commands::TestEndpoint { endpoint } => handle_test_endpoint(endpoint).await,
    };

    result
        .map(|s| json!({"jsonrpc": "2.0", "success": s}).to_string())
        .map_err(|e| SentinelError::Json(json!({"jsonrpc": "2.0", "error": e.to_string()})))
}
