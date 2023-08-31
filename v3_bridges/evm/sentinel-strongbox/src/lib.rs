#[cfg(target_os = "android")]
#[macro_use]
extern crate log;

#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "android")]
pub use self::android::{Error, Java_com_ptokenssentinelandroidapp_RustBridge_callCore};
