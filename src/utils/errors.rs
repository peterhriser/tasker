use std::{error::Error, fmt};

use crate::taskfile::TaskfileError;

pub type DynamicError = Box<dyn Error>;

fn print_error() {
    println!("Error!");
}
#[derive(Debug)]
pub enum UserFacingError {
    TaskfileDoesNotExist,    // Missing File Error
    TaskfileParseError,      // Invalid YAML Error
    TaskfileValidationError, // Serde Schema Validation Error
    TaskDoesNotExist,        // Referencing non-existing task
    TaskExecutionError,      // Command in task failed to run
    MissingContext,          // Referencing non-existing context
    MissingVariable,         // Variable value could not be found
}

impl std::error::Error for UserFacingError {}

impl fmt::Display for UserFacingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserFacingError::TaskfileDoesNotExist => write!(f, "Taskfile does not exist"),
            UserFacingError::TaskfileParseError => write!(f, "Taskfile could not be parsed"),
            UserFacingError::TaskfileValidationError => write!(f, "Taskfile is not valid"),
            UserFacingError::TaskDoesNotExist => write!(f, "Task does not exist"),
            UserFacingError::TaskExecutionError => write!(f, "Task could not be executed"),
            UserFacingError::MissingContext => write!(f, "Context does not exist"),
            UserFacingError::MissingVariable => write!(f, "Variable does not exist"),
        }
    }
}
impl From<TaskfileError> for UserFacingError {
    fn from(error: TaskfileError) -> Self {
        match error {
            TaskfileError::FileNotFound => UserFacingError::TaskfileDoesNotExist,
            TaskfileError::FileParseError => UserFacingError::TaskfileParseError,
        }
    }
}
fn handle_user_facing_error(error: UserFacingError) {
    match error {
        UserFacingError::TaskfileDoesNotExist => print_error(),
        UserFacingError::TaskfileParseError => print_error(),
        UserFacingError::TaskfileValidationError => print_error(),
        UserFacingError::TaskDoesNotExist => print_error(),
        UserFacingError::TaskExecutionError => print_error(),
        UserFacingError::MissingContext => print_error(),
        UserFacingError::MissingVariable => print_error(),
    }
}
