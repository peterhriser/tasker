use crate::cliargs::CliArgs;
use crate::run::TaskBuilder;
use crate::taskfile::Taskfile;
use crate::utils::errors::{ErrWithMessage, UserFacingError};
use clap::{ArgMatches, CommandFactory};
use std::path::PathBuf;

pub(crate) struct EntryPoint {
    initial_arg_matches: ArgMatches,
}
impl EntryPoint {
    pub fn new(cli_input: Option<Vec<&str>>) -> Result<EntryPoint, UserFacingError> {
        let matches = EntryPoint::get_matches(cli_input)?;
        Ok(EntryPoint {
            initial_arg_matches: matches,
        })
    }
    fn get_matches(cli_input: Option<Vec<&str>>) -> Result<ArgMatches, UserFacingError> {
        let cmd = CliArgs::command();
        return match cli_input {
            Some(args) => match cmd.try_get_matches_from(args) {
                Ok(matches) => Ok(matches),
                Err(e) => Err(UserFacingError::TaskfileDoesNotExist(ErrWithMessage {
                    code: "INVALID_TASKFILE_PATH".to_string(),
                    messages: vec![e.to_string()],
                })),
            },
            None => match cmd.try_get_matches() {
                Ok(matches) => Ok(matches),
                Err(e) => return Err(e.into()),
            },
        };
    }
    fn get_config_path(&self) -> Result<String, UserFacingError> {
        let config_path = match self.initial_arg_matches.get_one::<PathBuf>("config_path") {
            Some(fp) if fp.exists() => fp.to_string_lossy().to_string(),
            _ => {
                return Err(UserFacingError::TaskfileDoesNotExist(ErrWithMessage {
                    code: "INVALID_TASKFILE_PATH".to_string(),
                    messages: vec!["Taskfile does not exist".to_string()],
                }))
            }
        };
        Ok(config_path)
    }
    fn is_dry_run(&self) -> Result<bool, UserFacingError> {
        let dry_run = self.initial_arg_matches.get_one::<bool>("dry_run");
        match dry_run {
            Some(true) => Ok(true),
            _ => Ok(false),
        }
    }
    pub fn run(&self) -> Result<bool, UserFacingError> {
        let config_path = self.get_config_path()?;
        let config = Taskfile::new(config_path)?;
        let mut builder = TaskBuilder::new(config);
        let dry_run = self.is_dry_run()?;
        let runner = builder.create_task_runner(self.initial_arg_matches.to_owned())?;
        return match dry_run {
            true => {
                runner.print_commands();
                Ok(false)
            }
            false => {
                runner.execute_tasks()?;
                Ok(true)
            }
        };
    }
}

pub fn handle_result(result: Result<bool, UserFacingError>) {
    match result {
        Ok(true) => {
            println!("Task completed successfully");
        }
        Ok(false) => {
            println!("Task completed successfully (dry run)");
        }
        Err(e) => match e {
            UserFacingError::TaskfileDoesNotExist(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
            UserFacingError::TaskfileParseError(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
            UserFacingError::TaskExecutionError(_) => todo!(),
            UserFacingError::MissingArgError(_) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
            UserFacingError::TaskDoesNotExist(_) => {
                CliArgs::command().print_long_help().unwrap();
                std::process::exit(1);
            }
        },
    }
}
#[cfg(test)]
mod integration_tests {
    use crate::{cliargs::CliArgs, entrypoint::EntryPoint};
    use clap::{CommandFactory, FromArgMatches};

    #[test]
    fn test_entry_point() {
        let ep = EntryPoint::new(Some(vec![
            "tasker",
            "-c",
            "src/tests/Taskfile",
            "greet",
            "Peter",
        ]))
        .unwrap();
        let result = ep.run();
        assert!(result.is_ok())
    }
    #[test]
    fn test_from_arg_matches() {
        let initial_arg_matches = CliArgs::command().get_matches_from(vec![
            "tasker",
            "-c",
            "src/tests/Taskfile",
            "greet",
            "Peter",
        ]);
        let new_argmatches = CliArgs::from_arg_matches(&initial_arg_matches).unwrap();
        assert!(new_argmatches.config_path.exists());
    }
    #[test]
    fn test_new_empty() {
        let ep = EntryPoint::new(None);
        assert!(ep.is_err());
    }
    #[test]
    fn test_new_with_vec() {
        let ep = EntryPoint::new(Some(vec![
            "tasker",
            "-c",
            "src/tests/Taskfile",
            "greet",
            "Peter",
        ]));
        assert!(ep.is_ok());
    }
    #[test]
    fn test_missing_file() {
        let ep = EntryPoint {
            initial_arg_matches: CliArgs::command().get_matches_from(vec![
                "tasker",
                "-c",
                "src/tests/NotTaskfile",
                "greet",
                "Peter",
            ]),
        };
        let result = ep.run();
        assert!(result.is_err())
    }
    #[test]
    fn test_dry_run() {
        let ep = EntryPoint {
            initial_arg_matches: CliArgs::command().get_matches_from(vec![
                "tasker",
                "-c",
                "src/tests/Taskfile",
                "--dry-run",
                "greet",
                "Peter",
            ]),
        };
        assert!(ep.is_dry_run().unwrap());
        let ep = EntryPoint {
            initial_arg_matches: CliArgs::command().get_matches_from(vec![
                "tasker",
                "-c",
                "src/tests/Taskfile",
                "greet",
                "Peter",
            ]),
        };
        assert!(!ep.is_dry_run().unwrap());
    }
}
