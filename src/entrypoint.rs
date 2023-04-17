use crate::run::TaskBuilder;
use crate::taskfile::Taskfile;
use crate::utils::errors::{ErrWithMessage, UserFacingError};
use clap::{value_parser, ArgMatches, CommandFactory, Parser};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help(true), trailing_var_arg=true )]
struct CliArgs {
    #[arg(default_value = "Taskfile", short, long="config", help="path to file with task definitions", value_parser=value_parser!(PathBuf))]
    config_path: PathBuf,

    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        default_value = "help",
        help = "commands defined by Taskfile"
    )]
    task_info: Vec<String>,

    #[arg(
        short = 'x',
        long = "context",
        help = "execution context to load for command"
    )]
    context: Option<String>,
    #[arg(
        short,
        long,
        help = "print out the commands that would be run instead of executing them"
    )]
    dry_run: bool,
}

pub(crate) struct EntryPoint {
    initial_arg_matches: ArgMatches,
}
impl EntryPoint {
    pub fn new() -> EntryPoint {
        EntryPoint {
            initial_arg_matches: EntryPoint::get_matches(),
        }
    }
    fn get_matches() -> ArgMatches {
        CliArgs::command().get_matches()
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
        let runner = builder.create_task_runner(self.initial_arg_matches.to_owned());
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
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}