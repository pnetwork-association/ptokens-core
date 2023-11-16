use common::Bytes;
use common_sentinel::SentinelError;
use derive_more::Constructor;
use jni::{
    objects::{JObject, JValue},
    JNIEnv,
};

use crate::android::{
    check_and_handle_java_exceptions,
    type_aliases::{ByteArray, DataSensitivity},
};

#[derive(Constructor)]
pub struct Database<'a> {
    env: &'a JNIEnv<'a>,
    db_java_class: JObject<'a>,
}

impl<'a> Database<'a> {
    fn to_java_byte_array(&self, bs: &ByteArray) -> Result<JValue, SentinelError> {
        Ok(JValue::from(JObject::from(self.env.byte_array_from_slice(bs)?)))
    }

    pub fn start_transaction(&self) -> Result<(), SentinelError> {
        let print_exceptions = true;
        match self.env.call_method(self.db_java_class, "startTransaction", "()V", &[]) {
            Ok(_) => check_and_handle_java_exceptions(self.env, print_exceptions),
            Err(e) => self.handle_error(Err(e), print_exceptions),
        }
    }

    pub fn end_transaction(&self) -> Result<(), SentinelError> {
        let print_exceptions = true;
        match self.env.call_method(self.db_java_class, "endTransaction", "()V", &[]) {
            Ok(_) => check_and_handle_java_exceptions(self.env, print_exceptions),
            Err(e) => self.handle_error(Err(e), print_exceptions),
        }
    }

    pub fn delete(&self, k: &ByteArray) -> Result<(), SentinelError> {
        let print_exceptions = true;
        match self
            .env
            .call_method(self.db_java_class, "delete", "([B)V", &[self.to_java_byte_array(k)?])
        {
            Ok(_) => check_and_handle_java_exceptions(self.env, print_exceptions),
            Err(e) => self.handle_error(Err(e), print_exceptions),
        }
    }

    pub fn get(&self, k: &ByteArray, sensitivity: DataSensitivity) -> Result<Bytes, SentinelError> {
        // NOTE: Exceptions here are if there is no key in the db, which case should _always_
        // be handled by the rust core. Since we hit this case often (checking if blocks exist
        // etc), we don't want to pollute the logcat output by printing out many lines every
        // time we don't find an item in the db.
        let print_exceptions = false;
        let args = [
            self.to_java_byte_array(k)?,
            JValue::from(sensitivity.unwrap_or_default()),
        ];
        match self
            .env
            .call_method(self.db_java_class, "get", "([BB)[B", &args)
            .and_then(|ret| ret.l())
            .and_then(|j_value| self.env.convert_byte_array(j_value.into_inner()))
        {
            Ok(r) => check_and_handle_java_exceptions(self.env, print_exceptions).map(|_| r),
            Err(e) => self.handle_error(Err(e), print_exceptions),
        }
    }

    pub fn put(&self, k: &ByteArray, v: &ByteArray, sensitivity: Option<u8>) -> Result<(), SentinelError> {
        let print_exceptions = true;
        let args = [
            self.to_java_byte_array(k)?,
            self.to_java_byte_array(v)?,
            JValue::from(sensitivity.unwrap_or_default()),
        ];
        match self.env.call_method(self.db_java_class, "put", "([B[BB)V", &args) {
            Ok(_) => check_and_handle_java_exceptions(self.env, print_exceptions),
            Err(e) => self.handle_error(Err(e), print_exceptions),
        }
    }

    fn handle_error<T, E: Into<SentinelError> + std::fmt::Display>(
        &self,
        r: Result<T, E>,
        print_exceptions: bool,
    ) -> Result<T, SentinelError> {
        if let Err(e) = r {
            error!("{e}");
            if print_exceptions {
                self.env.exception_describe()?;
            };
            self.env.exception_clear()?;
            Err(e.into())
        } else {
            r.map_err(|e| e.into())
        }
    }
}
