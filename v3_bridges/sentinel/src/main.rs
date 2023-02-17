mod get_block;
mod get_receipts;
mod get_rpc_client;

use get_block::get_block;
use get_receipts::get_receipts;
use get_rpc_client::get_rpc_client;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    use simple_logger; // FIXME rm!
    simple_logger::init_with_level(log::Level::Info).unwrap(); // FIXME rm!

    let url = "ws://162.19.83.219:8546";
    let ws_client = get_rpc_client(url).await.unwrap();

    for i in 0..1 {
        let block = get_block(&ws_client, 16640611 + i).await.unwrap();
        let sub_mat = get_receipts(&ws_client, block).await.unwrap();
        let receipts_are_valid = sub_mat.receipts_are_valid().unwrap();
        println!("Receipts are valid: {receipts_are_valid}")
    }
}
// TODO use https://crates.io/crates/async-log for async logging when we have > 1 thread

// TODO use confy for easy config management, and toml files - better than json here.
// Write logs to a file, and watch that during running. (see vanilla apps for how that's done)
// pull the vanilla apps into this workspace too to share deps (rocksdb mostly)
// JSON-RPC spec:https://www.jsonrpc.org/specificationhttps://www.jsonrpc.org/specification https://www.jsonrpc.org/specification
