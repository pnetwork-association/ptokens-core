use std::{
    fs::File,
    io::{BufWriter, Write},
};

use common_sentinel::{get_sub_mat, Endpoints, NetworkId, SentinelError};
use tokio::time::{sleep, Duration};

pub async fn handle_get_sub_mat(block_num: u64, endpoint: String) -> Result<String, SentinelError> {
    debug!("getting sub mat...");
    let sleep_time = 30;
    let network_id = NetworkId::default();
    let endpoint = Endpoints::new(sleep_time, network_id, vec![endpoint.clone()]);
    let client = endpoint.get_first_ws_client().await?;
    let sleep_time = 30;
    let sub_mat = get_sub_mat(&client, block_num, sleep_time, &network_id).await?;
    let path = format!("./block-{block_num}.json");
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &sub_mat)?;
    writer.flush()?;
    // NOTE: sleep to let any connection teardown logs finish
    sleep(Duration::from_secs(1)).await;
    Ok(format!("file written to '{path}'"))
}
