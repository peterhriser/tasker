use std::fmt::{self, Display};

use crate::{run::errors::ExecutionError, taskfile::TaskfileError};

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
    TaskExecutionError(ErrWithMessage),   // Command in task failed to run
    MissingArgError(ErrWithMessage),      // Missing argument
    TaskDoesNotExist(ErrWithMessage),     // Task does not exist
}

impl std::error::Error for UserFacingError {}

impl fmt::Display for UserFacingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserFacingError::TaskfileDoesNotExist(e) => write!(f, "{}", e.to_string()),
            UserFacingError::TaskfileParseError(e) => write!(f, "{}", e.to_string()),
            UserFacingError::TaskExecutionError(e) => write!(f, "{}", e.to_string()),
            UserFacingError::MissingArgError(e) => write!(f, "{}", e.to_string()),
            UserFacingError::TaskDoesNotExist(e) => write!(f, "{}", e.to_string()),
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
impl From<ExecutionError> for UserFacingError {
    fn from(error: ExecutionError) -> Self {
        match error {
            ExecutionError::CommandFailed(mut e) => {
                e.add_to_stack("Command failed to execute".to_string());
                UserFacingError::TaskExecutionError(e)
            }
        }
    }
}
impl From<clap::Error> for UserFacingError {
    fn from(error: clap::error::Error) -> Self {
        match error.kind() {
            clap::error::ErrorKind::DisplayHelp => todo!(),
            clap::error::ErrorKind::DisplayVersion => todo!(),
            clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                UserFacingError::TaskDoesNotExist(ErrWithMessage {
                    code: "TASK_ERROR".to_string(),
                    messages: vec![error.to_string()],
                })
            }
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::utils::errors::{ErrWithMessage, UserFacingError};

    impl UserFacingError {
        pub fn add_to_error_stack(&mut self, message: String) {
            match self {
                UserFacingError::TaskfileDoesNotExist(e) => e.add_to_stack(message),
                UserFacingError::TaskfileParseError(e) => e.add_to_stack(message),
                // UserFacingError::TaskDoesNotExist(e) => e.add_to_stack(message),
                UserFacingError::TaskExecutionError(e) => e.add_to_stack(message),
                // UserFacingError::MissingContext(e) => e.add_to_stack(message),
                // UserFacingError::MissingVariable(e) => e.add_to_stack(message),
                _ => todo!(),
            }
        }
    }

    #[test]
    fn test_display_user_facing_error() {
        let error = UserFacingError::TaskfileDoesNotExist(ErrWithMessage {
            code: "FILE_ERROR".to_string(),
            messages: vec!["File does not exist".to_string()],
        });
        assert_eq!(
            error.to_string(),
            "\x1b[31mFILE_ERROR\x1b[0m: File does not exist"
        );
    }

    #[test]
    fn test_display_user_facing_error_with_stack() {
        let err = ErrWithMessage {
            code: "FILE_ERROR".to_string(),
            messages: vec!["File does not exist".to_string()],
        };
        let mut full_error = UserFacingError::TaskfileDoesNotExist(err);
        full_error.add_to_error_stack("Taskfile could not be found".to_string());
        assert_eq!(
            full_error.to_string(),
            "\x1b[31mFILE_ERROR\x1b[0m: Taskfile could not be found\n>    File does not exist"
        );
    }
    #[test]
    fn test_from_execution_error() {
        let error = UserFacingError::from(crate::taskfile::TaskfileError::FileParseError(
            ErrWithMessage {
                code: "FILE_ERROR".to_string(),
                messages: vec!["File does not exist".to_string()],
            },
        ));
        assert_eq!(
            error.to_string(),
            "\x1b[31mFILE_ERROR\x1b[0m: Taskfile encountered parsing issue\n>    File does not exist"
        );
    }
    #[test]
    fn test_from_taskfile_error() {
        let error = UserFacingError::from(crate::taskfile::TaskfileError::FileNotFound(
            ErrWithMessage {
                code: "FILE_ERROR".to_string(),
                messages: vec!["File does not exist".to_string()],
            },
        ));
        assert_eq!(
            error.to_string(),
            "\x1b[31mFILE_ERROR\x1b[0m: Taskfile could not be found\n>    File does not exist"
        );
    }
    #[test]
    fn test_from_taskfile_error_with_stack() {
        let err = ErrWithMessage {
            code: "FILE_ERROR".to_string(),
            messages: vec!["File does not exist".to_string()],
        };
        let mut full_error =
            UserFacingError::from(crate::taskfile::TaskfileError::FileNotFound(err));
        full_error.add_to_error_stack("Taskfile could not be found".to_string());
        assert_eq!(
            full_error.to_string(),
            "\x1b[31mFILE_ERROR\x1b[0m: Taskfile could not be found\n>    Taskfile could not be found\n>    File does not exist"
        );
    }
}
