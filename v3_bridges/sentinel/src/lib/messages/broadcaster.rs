use crate::UserOps;

#[derive(Debug)]
pub enum BroadcasterMessages {
    CancelUserOps(UserOps),
}
