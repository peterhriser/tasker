use crate::utils::errors::ErrWithMessage;

#[derive(Debug)]
pub enum ExecutionError {
    CommandFailed(ErrWithMessage),
}
impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExecutionError::CommandFailed(e) => write!(f, "Command failed to execute:\n    {}", e),
        }
    }
}
impl From<std::io::Error> for ExecutionError {
    fn from(e: std::io::Error) -> Self {
        ExecutionError::CommandFailed(ErrWithMessage {
            code: "COMMAND_ERROR".to_string(),
            messages: vec![format!(
                "Command failed to execute with message: {}",
                e.to_string()
            )],
        })
    }
}
impl From<std::env::VarError> for ExecutionError {
    fn from(e: std::env::VarError) -> Self {
        ExecutionError::CommandFailed(ErrWithMessage {
            code: "COMMAND_ERROR".to_string(),
            messages: vec![format!(
                "Command failed to execute with message: {}",
                e.to_string()
            )],
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Error;

    #[test]
    fn test_display_execution_error() {
        let error = ExecutionError::CommandFailed(ErrWithMessage {
            code: "COMMAND_ERROR".to_string(),
            messages: vec!["Command failed to execute".to_string()],
        });
        assert_eq!(
            error.to_string(),
            "Command failed to execute:\n    \u{1b}[31mCOMMAND_ERROR\u{1b}[0m: Command failed to execute"
        );
    }

    #[test]
    fn test_display_execution_error_from_io_error() {
        let error = ExecutionError::from(Error::new(
            std::io::ErrorKind::NotFound,
            "Command not found",
        ));
        assert_eq!(
            error.to_string(),
            "Command failed to execute:\n    \u{1b}[31mCOMMAND_ERROR\u{1b}[0m: Command failed to execute with message: Command not found"
        );
    }

    #[test]
    fn test_display_execution_error_from_env_error() {
        let error = ExecutionError::from(std::env::VarError::NotPresent);
        assert_eq!(
            error.to_string(),
            "Command failed to execute:\n    \u{1b}[31mCOMMAND_ERROR\u{1b}[0m: Command failed to execute with message: environment variable not found"
        );
    }
    #[test]
    fn test_h() {}
}
