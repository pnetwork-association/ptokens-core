#[cfg(not(target_os = "android"))]
compile_error!("this repo targets android only - please compile with `--target=aarch64-linux-android` flag");

#[cfg(target_os = "android")]
#[macro_use]
extern crate log;

#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "android")]
pub use self::android::{
    Java_com_ptokenssentinelandroidapp_RustBridge_callCore,
    CoreError,
};
