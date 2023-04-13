use crate::utils::errors::ErrWithMessage;

#[derive(Debug)]
pub enum ExecutionError {
    CommandFailed(ErrWithMessage),
    CommandNotFound(ErrWithMessage),
}
impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExecutionError::CommandFailed(e) => write!(f, "Command failed to execute:\n    {}", e),
            ExecutionError::CommandNotFound(e) => write!(f, "Command not found:\n    {}", e),
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
