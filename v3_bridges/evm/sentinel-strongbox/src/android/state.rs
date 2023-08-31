use super::Database;
use common_sentinel::{SentinelError, WebSocketMessages};
use jni::{
    objects::{JObject, JString, JValue},
    JNIEnv,
};
use super::type_aliases::{Bytes, JavaPointer};

pub struct State<'a> {
    db: Database<'a>,
    env: &'a JNIEnv<'a>,
    msg: WebSocketMessages,
}

impl<'a> State<'a> {
    pub fn new(
        env: &'a JNIEnv<'a>,
        db_java_class: JObject<'a>,
        input: JString
    ) -> Result<Self, SentinelError> {
        let db = Database::new(env, db_java_class);
        let input_string: String = env.get_string(input)?.into();
        let msg = WebSocketMessages::try_from(input_string)?;
        Ok(State { env, db, msg })
    }

    pub fn msg(&self) -> &WebSocketMessages {
        &self.msg
    }

    pub fn db(&self) -> &Database<'a> {
        &self.db
    }

    pub fn to_return_value_pointer(&self, s: &str) -> Result<*mut JavaPointer, SentinelError> {
        // TODO try into?
        Ok(self.to_jstring(s)?.into_inner())
    }

    fn parse_jstring(&self, input: JString) -> Result<String, SentinelError> {
        Ok(self.env.get_string(input)?.into())
    }

    fn to_jstring(&self, s: &str) -> Result<JString<'_>, SentinelError> {
        Ok(self.env.new_string(s)?)
    }

    fn db_java_class(&self) -> JObject<'a> {
        self.db.db_java_class()
    }

    fn call_callback(&self) -> Result<(), SentinelError> {
        match self.env.call_static_method(self.db_java_class(), "callback", "()V", &[]) {
            Ok(_) => Ok(()),
            Err(e) => {
                self.env.exception_describe()?;
                self.env.exception_clear()?;
                Err(e.into())
            },
        }
    }
}
