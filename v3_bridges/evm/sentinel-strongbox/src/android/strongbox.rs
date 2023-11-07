use derive_getters::Getters;
use derive_more::Constructor;
use common_sentinel::SentinelError;
use thiserror::Error;

use super::State;
use jni::{
    objects::{JObject, JValue},
    JNIEnv,
};

#[derive(Constructor, Getters)]
pub struct Strongbox<'a> {
    env: &'a JNIEnv<'a>,
    strongbox_java_class: JObject<'a>,
}

impl<'a> Strongbox<'a> {
    pub fn check_keystore_is_initialized(&self) -> Result<bool, SentinelError> {
        debug!("checking strongbox keystore is initialized...");
        match self.env().call_method(self.strongbox_java_class, "keystoreIsInitialized", "()Z", &[]) {
            Ok(r) => {
                self.env().exception_describe().expect("this not to fail");
                self.env().exception_clear().expect("this not to fail");
                // NOTE: This following obscenity is getting the return value (a bool) from the function call
                // via jni. `z` here being the encoded return value. See here for more info:
                // https://docs.oracle.com/javase/7/docs/technotes/guides/jni/spec/types.html#wp276
                r.z().map_err(|e| e.into())
            },
            Err(e) => {
                error!("{e}");
                Err(e.into())
            },
        }
    }

    pub fn sign_with_attestation_key(&self) -> Result<Vec<u8>, SentinelError> {
        todo!("this");
    }

    pub fn get_attestation_certificate(&self) -> Result<String, SentinelError> {
        todo!("this");
    }

    pub fn initialize_keystore(&self) -> Result<(), SentinelError> {
        todo!("this");
    }

}
