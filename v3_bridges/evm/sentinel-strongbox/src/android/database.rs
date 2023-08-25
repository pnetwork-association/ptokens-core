use super::{
    type_aliases::{ByteArray, Bytes, DataSensitivity, JavaPointer},
    CoreError,
};
use derive_more::Constructor;
use jni::{
    objects::{JObject, JString, JValue},
    JNIEnv,
};

#[derive(Constructor)]
pub struct Database<'a> {
    env: &'a JNIEnv<'a>,
    db_java_class: JObject<'a>,
}

impl Database<'_> {
    pub fn parse_input(&self, input: JString) -> Result<String, CoreError> {
        Ok(self.env.get_string(input)?.into())
    }

    fn to_jstring(&self, s: &str) -> Result<JString<'_>, CoreError> {
        Ok(self.env.new_string(s)?)
    }

    pub fn to_return_value_pointer(&self, s: &str) -> Result<*mut JavaPointer, CoreError> {
        Ok(self.to_jstring(s)?.into_inner())
    }

    pub fn call_callback(&self) -> Result<(), CoreError> {
        match self
            .env
            .call_static_method(self.db_java_class, "callback", "()V", &[])
        {
            Ok(_) => Ok(()),
            Err(e) => {
                self.env.exception_describe()?;
                self.env.exception_clear()?;
                Err(e.into())
            }
        }
    }

    fn to_java_byte_array(&self, bs: &ByteArray) -> Result<JValue, CoreError> {
        Ok(JValue::from(JObject::from(
            self.env.byte_array_from_slice(bs)?,
        )))
    }

    pub fn start_transaction(&self) -> Result<(), CoreError> {
        match self
            .env
            .call_method(self.db_java_class, "startTransaction", "()V", &[])
        {
            Ok(_) => Ok(()),
            Err(e) => self.handle_error(Err(e)),
        }
    }

    pub fn end_transaction(&self) -> Result<(), CoreError> {
        match self
            .env
            .call_method(self.db_java_class, "endTransaction", "()V", &[])
        {
            Ok(_) => Ok(()),
            Err(e) => self.handle_error(Err(e)),
        }
    }

    pub fn delete(&self, k: &ByteArray) -> Result<(), CoreError> {
        match self.env.call_method(
            self.db_java_class,
            "delete",
            "([B)V",
            &[self.to_java_byte_array(k)?],
        ) {
            Ok(_) => Ok(()),
            Err(e) => self.handle_error(Err(e)),
        }
    }

    pub fn get(&self, k: &ByteArray, sensitivity: DataSensitivity) -> Result<Bytes, CoreError> {
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
            Ok(r) => Ok(r),
            Err(e) => self.handle_error(Err(e)),
        }
    }

    pub fn put(
        &self,
        k: &ByteArray,
        v: &ByteArray,
        sensitivity: Option<u8>,
    ) -> Result<(), CoreError> {
        let args = [
            self.to_java_byte_array(k)?,
            self.to_java_byte_array(v)?,
            JValue::from(sensitivity.unwrap_or_default()),
        ];
        match self
            .env
            .call_method(self.db_java_class, "put", "([B[BB)V", &args)
        {
            Ok(_) => Ok(()),
            Err(e) => self.handle_error(Err(e)),
        }
    }

    fn handle_error<T, E: Into<CoreError> + std::fmt::Display>(
        &self,
        r: Result<T, E>,
    ) -> Result<T, CoreError> {
        if let Err(e) = r {
            error!("{e}");
            self.env.exception_describe()?;
            self.env.exception_clear()?;
            Err(e.into())
        } else {
            r.map_err(|e| e.into())
        }
    }
}
