use std::result::Result;

use common_sentinel::{Endpoints, NetworkId, SentinelError};
use tokio::time::{sleep, Duration};

pub async fn handle_test_endpoint(endpoint: String) -> Result<String, SentinelError> {
    debug!("handling test endpoint");
    let sleep_time = 30;
    let e = Endpoints::new(sleep_time, NetworkId::default(), vec![endpoint.clone()]);
    let r = e.check_endpoint(sleep_time).await;
    // NOTE: sleep to let any connection teardown logs finish
    sleep(Duration::from_millis(1000)).await;
    r.map(|_| format!("{endpoint} is working as expected"))
}
