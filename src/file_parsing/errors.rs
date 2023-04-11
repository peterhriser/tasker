use std::{error::Error, fmt};

#[derive(Debug)]
pub enum TaskfileError {
    FileNotFound,
    FileParseError,
}
impl Error for TaskfileError {}
impl fmt::Display for TaskfileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskfileError::FileNotFound => write!(f, "Taskfile does not exist"),
            TaskfileError::FileParseError => write!(f, "Taskfile could not be parsed"),
        }
    }
}
impl From<std::io::Error> for TaskfileError {
    fn from(_: std::io::Error) -> Self {
        TaskfileError::FileNotFound
    }
}
impl From<serde_yaml::Error> for TaskfileError {
    fn from(_: serde_yaml::Error) -> Self {
        TaskfileError::FileParseError
    }
}
