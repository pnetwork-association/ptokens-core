use super::Responder;

#[derive(Debug)]
pub enum CoreAccessorMessages {
    GetHostLatestBlockNumber(Responder<u64>),
    GetNativeLatestBlockNumber(Responder<u64>),
}
