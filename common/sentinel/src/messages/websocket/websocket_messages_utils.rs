use super::WebSocketMessagesError;

pub fn check_num_args(expected_n: usize, args: Vec<String>) -> Result<Vec<String>, WebSocketMessagesError> {
    let n = args.len();
    if n < expected_n {
        Err(WebSocketMessagesError::NotEnoughArgs {
            got: n,
            expected: expected_n,
            args,
        })
    } else {
        Ok(args)
    }
}
