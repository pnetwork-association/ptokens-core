use common_sentinel::SentinelError;
use jni::JNIEnv;

pub(crate) fn check_and_handle_java_exceptions<'a>(
    env: &'a JNIEnv<'a>,
    print_exceptions: bool,
) -> Result<(), SentinelError> {
    if matches!(env.exception_check(), Ok(true)) {
        if print_exceptions {
            env.exception_describe()?;
        };
        env.exception_clear()?;
        Err(SentinelError::JavaExceptionOccurred)
    } else {
        Ok(())
    }
}
