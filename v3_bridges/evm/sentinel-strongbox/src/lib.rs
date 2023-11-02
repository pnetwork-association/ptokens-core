#![cfg(target_os = "android")]

#[macro_use]
extern crate log;

#[macro_use]
extern crate common;

mod android;

pub use self::android::Java_com_ptokenssentinelandroidapp_RustBridge_callCore;
