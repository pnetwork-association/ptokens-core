use common_sentinel::SentinelError;

use crate::android::{check_and_handle_java_exceptions, constants::PRINT_JAVA_ERRORS, Database};

impl<'a> Database<'a> {
    pub fn drop_db(&self) -> Result<(), SentinelError> {
        debug!("dropping db...");

        let env = self.env();
        match env.call_method(*self.db_java_class(), "drop", "()V", &[]) {
            Err(e) => self.handle_error(Err(e), PRINT_JAVA_ERRORS),
            Ok(_) => {
                check_and_handle_java_exceptions(env, PRINT_JAVA_ERRORS)?;
                Ok(())
            },
        }
    }
}
