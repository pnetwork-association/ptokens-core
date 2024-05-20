use common_sentinel::{DbIntegrity, SentinelError};
use jni::objects::JString;

use super::DbJsonResponse;
use crate::android::{check_and_handle_java_exceptions, constants::PRINT_JAVA_ERRORS, Database};

// NOTE: Unlike other methods in the java db interface, these two return a json string which needs
// parsing in order to know the integrity of the database.

impl<'a> Database<'a> {
    pub fn start_transaction(&self) -> Result<(), SentinelError> {
        debug!("staring db transaction...");

        let env = self.env();
        match env.call_method(*self.db_java_class(), "startTransaction", "()Ljava/lang/String;", &[]) {
            Err(e) => self.handle_error(Err(e), PRINT_JAVA_ERRORS),
            Ok(r) => {
                check_and_handle_java_exceptions(env, PRINT_JAVA_ERRORS)?;
                let jstring: JString = r.l()?.into();
                let s: String = env.get_string(jstring)?.into();
                let db_json_response = DbJsonResponse::try_from(s.as_ref())?;
                let db_integrity = DbIntegrity::try_from(db_json_response)?;

                debug!("db integrity: {db_integrity}");
                if db_integrity.is_valid() {
                    Ok(())
                } else {
                    Err(SentinelError::InvalidDbIntegrity(db_integrity))
                }
            },
        }
    }

    // FIXME The android/java side can also return a none-db-integrity related error here, if
    // there's already an end transaction in process. TODO Handle that case.
    pub fn end_transaction(&self) -> Result<(), SentinelError> {
        debug!("ending db transaction...");

        let env = self.env();
        match env.call_method(*self.db_java_class(), "endTransaction", "()Ljava/lang/String;", &[]) {
            Err(e) => self.handle_error(Err(e), PRINT_JAVA_ERRORS),
            Ok(r) => {
                check_and_handle_java_exceptions(env, PRINT_JAVA_ERRORS)?;
                let jstring: JString = r.l()?.into();
                let s: String = env.get_string(jstring)?.into();
                let db_json_response = DbJsonResponse::try_from(s.as_ref())?;
                let db_integrity = DbIntegrity::try_from(db_json_response)?;

                debug!("db integrity: {db_integrity}");
                if db_integrity.is_valid() {
                    Ok(())
                } else {
                    Err(SentinelError::InvalidDbIntegrity(db_integrity))
                }
            },
        }
    }
}
