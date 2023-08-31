mod state;
mod database;
mod type_aliases;

use common_sentinel::{
    SentinelError,
    WebSocketMessages,
};
use jni::{
    objects::{JClass, JObject, JString},
    sys::jstring,
    JNIEnv,
};

use self::{
    state::State,
    database::Database,
    type_aliases::{Bytes, JavaPointer},
};

fn call_core_inner(env: JNIEnv<'_>, db_java_class: JObject, input: JString) -> Result<*mut JavaPointer, SentinelError> {
    let state = State::new(&env, db_java_class, input)?;
    state.to_return_value_pointer("some str")
}

// FIXME Important! The java db is NOT single threaded! We need a shim here to intercept errors
// (whilst we still have the ability to callback to the java db stuff), and in the case of errors
// here where the db tx is never ended (as it shouldn't be in the case of errors), we need to call
// something to clean up the flags in the java db impl to allow new txs to be started.

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Java_com_ptokenssentinelandroidapp_RustBridge_callCore(
    env: JNIEnv,
    _class: JClass,
    db_java_class: JObject,
    input: JString,
) -> jstring {
    call_core_inner(env, db_java_class, input)
        .map_err(|e| e.to_string())
        .expect("this not to panic")
}
