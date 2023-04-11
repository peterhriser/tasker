use std::{error::Error, fmt};

use crate::utils::errors::ErrWithMessage;
#[derive(Debug)]
pub enum TaskfileError {
    FileNotFound(ErrWithMessage),
    FileParseError(ErrWithMessage),
}
impl Error for TaskfileError {}
impl fmt::Display for TaskfileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskfileError::FileNotFound(e) => write!(f, "Taskfile does not exist:\n    {}", e),
            TaskfileError::FileParseError(e) => write!(f, "Taskfile Parsing Error:\n    {}", e),
        }
    }
}
impl From<std::io::Error> for TaskfileError {
    fn from(e: std::io::Error) -> Self {
        TaskfileError::FileNotFound(ErrWithMessage {
            code: "FILE_ERROR".to_string(),
            messages: vec![e.to_string()],
        })
    }
}
impl From<serde_yaml::Error> for TaskfileError {
    fn from(e: serde_yaml::Error) -> Self {
        TaskfileError::FileParseError(ErrWithMessage {
            code: "YAML_PARSE_ERROR".to_string(),
            messages: vec![e.to_string()],
        })
    }
}
