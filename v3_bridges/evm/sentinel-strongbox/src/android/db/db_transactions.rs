use common_sentinel::SentinelError;

use crate::android::{check_and_handle_java_exceptions, Database};

impl<'a> Database<'a> {
    pub fn start_transaction(&self) -> Result<(), SentinelError> {
        let print_exceptions = true;
        match self
            .env()
            .call_method(*self.db_java_class(), "startTransaction", "()V", &[])
        {
            Ok(_) => check_and_handle_java_exceptions(self.env(), print_exceptions),
            Err(e) => self.handle_error(Err(e), print_exceptions),
        }
    }

    pub fn end_transaction(&self) -> Result<(), SentinelError> {
        let print_exceptions = true;
        match self
            .env()
            .call_method(*self.db_java_class(), "endTransaction", "()V", &[])
        {
            Ok(_) => check_and_handle_java_exceptions(self.env(), print_exceptions),
            Err(e) => self.handle_error(Err(e), print_exceptions),
        }
    }
}
