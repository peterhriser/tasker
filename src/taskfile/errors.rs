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

#[cfg(test)]
mod test {
    #[test]
    fn test_from_io_error() {
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "File does not exist");
        let taskfile_error = super::TaskfileError::from(error);
        assert_eq!(
            taskfile_error.to_string(),
            "Taskfile does not exist:\n    \u{1b}[31mFILE_ERROR\u{1b}[0m: File does not exist"
        )
    }
    #[test]
    fn test_from_yaml_error() {
        let error: serde_yaml::Result<Vec<String>> = serde_yaml::from_str("invalid yaml");
        let taskfile_error = super::TaskfileError::from(error.unwrap_err());
        assert_eq!(
            taskfile_error.to_string(),
            "Taskfile Parsing Error:\n    \u{1b}[31mYAML_PARSE_ERROR\u{1b}[0m: invalid type: string \"invalid yaml\", expected a sequence"
        )
    }
}
