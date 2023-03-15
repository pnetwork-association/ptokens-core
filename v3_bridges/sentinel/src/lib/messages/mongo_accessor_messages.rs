#![allow(unused)] // FIXME rm!
use common::BridgeSide;
use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{Responder, SentinelError};

#[derive(Debug)]
pub enum MongoAccessorMessages {}
