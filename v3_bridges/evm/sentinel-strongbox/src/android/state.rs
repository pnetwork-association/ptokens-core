use common_sentinel::{SentinelError, WebSocketMessagesEncodable};
use derive_getters::Getters;
use jni::{
    objects::{JObject, JString},
    JNIEnv,
};
use serde::Serialize;

use super::{strongbox::Strongbox, type_aliases::JavaPointer, Database};

#[derive(Serialize, Getters)]
pub struct State<'a> {
    #[serde(skip_serializing)]
    db: Database<'a>,

    #[serde(skip_serializing)]
    env: &'a JNIEnv<'a>,

    #[serde(skip_serializing)]
    strongbox: Strongbox<'a>,

    #[serde(skip_serializing_if = "Option::is_none")]
    res: Option<WebSocketMessagesEncodable>,

    // NOTE: We need the state struct serializable to work with debug signature macros, however we
    // definitely don't need the db pointer nor JNI env nor empty return values serialized, hence the
    // above skips.
    msg: WebSocketMessagesEncodable,
}

impl<'a> State<'a> {
    pub fn add_response(mut self, r: WebSocketMessagesEncodable) -> Self {
        self.res = Some(r);
        self
    }

    pub fn new(
        env: &'a JNIEnv<'a>,
        strongbox_java_class: JObject<'a>,
        db_java_class: JObject<'a>,
        input: JString,
    ) -> Result<Self, SentinelError> {
        let db = Database::new(env, db_java_class);
        let input_string: String = env.get_string(input)?.into();
        let msg = WebSocketMessagesEncodable::try_from(input_string)?;
        let strongbox = Strongbox::new(env, strongbox_java_class);
        Ok(State {
            env,
            msg,
            db,
            res: None,
            strongbox,
        })
    }

    pub fn to_response(&self) -> Result<*mut JavaPointer, SentinelError> {
        let s: String = match self.res.clone() {
            // FIXME rm this clone
            Some(r) => r.try_into(),
            None => {
                // NOTE: We haven't error, but we also don't have a response for some reason
                // FIXME Should this be an error?
                warn!("no response in state");
                WebSocketMessagesEncodable::Null.try_into()
            },
        }?;
        self.to_return_value_pointer(&s)
    }

    pub fn to_return_value_pointer(&self, s: &str) -> Result<*mut JavaPointer, SentinelError> {
        // TODO try into?
        Ok(self.to_jstring(s)?.into_inner())
    }

    fn to_jstring(&self, s: &str) -> Result<JString<'_>, SentinelError> {
        Ok(self.env.new_string(s)?)
    }
}
