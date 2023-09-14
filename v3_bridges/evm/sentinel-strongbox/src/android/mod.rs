use std::panic;
mod database;
mod handle_websocket_message;
mod handlers;
mod state;
mod type_aliases;

use android_logger::Config;
use common_sentinel::{SentinelError, WebSocketMessagesEncodable, WebSocketMessagesError};
use jni::{
    objects::{JClass, JObject, JString},
    sys::{jint, jstring},
    JNIEnv,
    JavaVM,
};
use log::LevelFilter;

use self::{
    database::Database,
    handle_websocket_message::handle_websocket_message,
    state::State,
    type_aliases::JavaPointer,
};

fn call_core_inner(
    env: &JNIEnv<'_>,
    db_java_class: JObject,
    input: JString,
) -> Result<*mut JavaPointer, SentinelError> {
    State::new(env, db_java_class, input)
        .and_then(handle_websocket_message)
        .and_then(|state| state.to_response())
}

// FIXME Important! The java db is NOT single threaded! We need a shim here to intercept errors
// (whilst we still have the ability to callback to the java db stuff), and in the case of errors
// here where the db tx is never ended (as it shouldn't be in the case of errors), we need to call
// something to clean up the flags in the java db impl to allow new txs to be started.

#[no_mangle]
#[allow(non_snake_case, improper_ctypes_definitions)]
pub extern "system" fn JNI_OnLoad(_vm: JavaVM) -> jint {
    android_logger::init_once(Config::default().with_max_level(LevelFilter::Debug));
    info!("android logger loaded");
    jni::sys::JNI_VERSION_1_6
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Java_com_ptokenssentinelandroidapp_RustBridge_callCore(
    env: JNIEnv,
    _class: JClass,
    db_java_class: JObject,
    input: JString,
) -> jstring {
    let result = panic::catch_unwind(|| {
        match call_core_inner(&env, db_java_class, input) {
            Ok(r) => r,
            Err(e) => {
                error!("{e}");

                // First we need to cancel the db transaction...
                match env.call_method(db_java_class, "cancelTransaction", "()V", &[]) {
                    Ok(_) => {
                        env.exception_describe().expect("this not to fail"); // FIXME
                        env.exception_clear().expect("this not to fail"); // FIXME How to handle if an exception
                                                                          // occurred here? Do we return anything?
                    },
                    Err(e) => {
                        // FIXME check for java exceptions!
                        error!("{e}");
                        let r: String = match WebSocketMessagesEncodable::Error(WebSocketMessagesError::JavaDb(
                            "could not cancel db tx".into(),
                        ))
                        .try_into()
                        {
                            Ok(s) => s,
                            Err(e) => {
                                error!("{e}");
                                format!("{e}")
                            },
                        };
                        return env
                            .new_string(r.to_string())
                            .expect("this should not fail")
                            .into_inner();
                    },
                };

                // NOTE: Now we need to handle whatever went wrong. Lets wrap the error in an encodable websocket
                // message and return it to the caller.
                let r: String = match WebSocketMessagesEncodable::Error(e.into()).try_into() {
                    Ok(s) => s,
                    Err(e) => {
                        error!("{e}");
                        format!("{e}")
                    },
                };
                env.new_string(r.to_string())
                    .expect("this should not fail")
                    .into_inner()
            },
        }
    });
    match result {
        Ok(r) => r,
        Err(e) => {
            error!("something panicked: {e:?}");
            env.new_string(format!("{e:?}")) // TODO wrap in encodeable error?
                .expect("this should not fail")
                .into_inner()
        },
    }
}
