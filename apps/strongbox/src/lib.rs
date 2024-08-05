#![cfg(target_os = "android")]

#[macro_use]
extern crate log;

#[macro_use]
extern crate common;

mod android;

pub use self::android::{
    Java_proofcastlabs_tee_MainActivity_callCore,
    Java_proofcastlabs_tee_logging_RustLogger_log,
};
