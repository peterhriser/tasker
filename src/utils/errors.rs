use std::fmt::{self, Display};

use crate::taskfile::TaskfileError;

#[derive(Debug)]
// todo: make this have a trace
pub struct ErrWithMessage {
    pub code: String,
    pub messages: Vec<String>,
}
impl ErrWithMessage {
    pub fn add_to_stack(&mut self, message: String) {
        self.messages.push(message);
    }
}
impl Display for ErrWithMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result_to_be = String::new();
        let mut iter = self.messages.clone();
        iter.reverse();
        let mut iter = iter.iter();
        result_to_be.push_str(&format!(
            "\x1b[31m{}\x1b[0m: {}",
            self.code,
            iter.next().unwrap()
        ));
        for message in iter {
            result_to_be.push_str(&format!("\n>    {}", message));
        }
        write!(f, "{}", result_to_be)
    }
}
#[derive(Debug)]
pub enum UserFacingError {
    TaskfileDoesNotExist(ErrWithMessage), // Missing File Error
    TaskfileParseError(ErrWithMessage),   // Invalid YAML Error
    TaskDoesNotExist(ErrWithMessage),     // Referencing non-existing task
    TaskExecutionError(ErrWithMessage),   // Command in task failed to run
    MissingContext(ErrWithMessage),       // Referencing non-existing context
    MissingVariable(ErrWithMessage),      // Variable value could not be found
}

impl std::error::Error for UserFacingError {}

impl fmt::Display for UserFacingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserFacingError::TaskfileDoesNotExist(e) => write!(f, "{}", e.to_string()),
            UserFacingError::TaskfileParseError(e) => write!(f, "{}", e.to_string()),
            UserFacingError::TaskDoesNotExist(e) => write!(f, "{}", e.to_string()),
            UserFacingError::TaskExecutionError(e) => write!(f, "{}", e.to_string()),
            UserFacingError::MissingContext(e) => write!(f, "{}", e.to_string()),
            UserFacingError::MissingVariable(e) => write!(f, "{}", e.to_string()),
        }
    }
}
impl From<TaskfileError> for UserFacingError {
    fn from(error: TaskfileError) -> Self {
        match error {
            TaskfileError::FileNotFound(mut e) => {
                e.add_to_stack("Taskfile could not be found".to_string());
                UserFacingError::TaskfileDoesNotExist(e)
            }
            TaskfileError::FileParseError(mut e) => {
                e.add_to_stack("Taskfile encountered parsing issue".to_string());
                UserFacingError::TaskfileParseError(e)
            }
        }
    }
}
pub fn handle_user_facing_error(error: UserFacingError) {
    match error {
        _ => println!("{}", error.to_string()),
    }
}
