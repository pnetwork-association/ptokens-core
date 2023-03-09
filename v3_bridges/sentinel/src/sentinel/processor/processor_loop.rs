use std::result::Result;

use lib::SentinelError;
use tokio::{
    sync::broadcast::Receiver,
    time::{sleep, Duration},
};

pub async fn processor_loop(mut rx: Receiver<bool>) -> Result<(), SentinelError> {
    info!("Starting processor loop...");
    let log_prefix = "processor";
    let mut i = 0;

    'main: loop {
        if let Ok(boolean) = rx.try_recv() {
            info!("{log_prefix} broadcast message received!");
            if boolean == true {
                warn!("{log_prefix} shutting down!");
                return Ok(());
            }
        }

        info!("Processor loop #{i}");
        sleep(Duration::from_millis(5_000)).await;
        i += 1;
        continue 'main;
    }
}
