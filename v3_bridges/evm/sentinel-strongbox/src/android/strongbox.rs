use common_sentinel::SentinelError;
use derive_getters::Getters;
use derive_more::Constructor;
use jni::{
    objects::{JObject, JString, JValue},
    JNIEnv,
};

use crate::android::{check_and_handle_java_exceptions, constants::PRINT_JAVA_ERRORS};

#[derive(Constructor, Getters)]
pub struct Strongbox<'a> {
    env: &'a JNIEnv<'a>,
    strongbox_java_class: JObject<'a>,
}

impl<'a> Strongbox<'a> {
    fn check_keystore_is_initialized(&self) -> Result<bool, SentinelError> {
        debug!("checking strongbox keystore is initialized...");
        match self
            .env()
            .call_method(self.strongbox_java_class, "keystoreIsInitialized", "()Z", &[])
        {
            Ok(r) => {
                check_and_handle_java_exceptions(self.env, PRINT_JAVA_ERRORS)?;
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

    pub fn get_attestation_signature(&self, bytes: Vec<u8>) -> Result<Vec<u8>, SentinelError> {
        debug!("getting attestation signature...");

        if !matches!(self.check_keystore_is_initialized(), Ok(true)) {
            self.initialize_keystore()?;
        };

        let bytes_java_param = JValue::from(JObject::from(self.env.byte_array_from_slice(&bytes)?));

        match self
            .env()
            .call_method(self.strongbox_java_class, "signWithAttestationKey", "([B)[B", &[
                bytes_java_param,
            ]) {
            Ok(r) => {
                check_and_handle_java_exceptions(self.env, PRINT_JAVA_ERRORS)?;
                let jobject = r.l()?; // NOTE: See above for the strange fxn call stuff
                let s: Vec<u8> = self.env.convert_byte_array(*jobject)?;
                Ok(s)
            },
            Err(e) => {
                error!("{e}");
                Err(e.into())
            },
        }
    }

    pub fn get_attestation_certificate(&self) -> Result<String, SentinelError> {
        debug!("getting attestation certificate...");

        if !matches!(self.check_keystore_is_initialized(), Ok(true)) {
            self.initialize_keystore()?;
        };

        match self.env().call_method(
            self.strongbox_java_class,
            "getCertificateAttestation",
            "()Ljava/lang/String;",
            &[],
        ) {
            Ok(r) => {
                check_and_handle_java_exceptions(self.env, PRINT_JAVA_ERRORS)?;
                let jstring: JString = r.l()?.into(); // NOTE: See above for the strange fxn call stuff
                let s: String = self.env.get_string(jstring)?.into();
                Ok(s)
            },
            Err(e) => {
                error!("{e}");
                Err(e.into())
            },
        }
    }

    fn initialize_keystore(&self) -> Result<(), SentinelError> {
        if matches!(self.check_keystore_is_initialized(), Ok(true)) {
            debug!("keystore already initialized!");
            Ok(())
        } else {
            debug!("initializing keystore...");
            match self
                .env()
                .call_method(self.strongbox_java_class, "initializeKeystore", "()V", &[])
            {
                Ok(_) => {
                    check_and_handle_java_exceptions(self.env, PRINT_JAVA_ERRORS)?;
                    Ok(())
                },
                Err(e) => {
                    error!("{e}");
                    Err(e.into())
                },
            }
        }
    }
}
