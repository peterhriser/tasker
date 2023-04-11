mod run;
mod taskfile;
mod tests;
mod utils;

use crate::{taskfile::Taskfile, utils::errors::handle_user_facing_error};
use clap::{value_parser, ArgMatches, CommandFactory, Parser};
use run::TaskBuilder;
use std::path::PathBuf;
use utils::errors::{ErrWithMessage, UserFacingError};

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

fn run_from_matches(initial_arg_matches: ArgMatches) -> Result<bool, UserFacingError> {
    let config_path = match initial_arg_matches.get_one::<PathBuf>("config_path") {
        Some(fp) if fp.exists() => fp.to_string_lossy().to_string(),
        _ => {
            return Err(UserFacingError::TaskfileDoesNotExist(ErrWithMessage {
                code: "INVALID_TASKFILE_PATH".to_string(),
                messages: vec!["Taskfile does not exist".to_string()],
            }))
        }
    };
    let config = Taskfile::new(config_path)?;

    let mut builder = TaskBuilder::new(config);
    let dry_run = initial_arg_matches.get_one::<bool>("dry_run");
    let runner = builder.create_task_runner(initial_arg_matches.to_owned());
    return match dry_run {
        Some(true) => {
            runner.print_commands();
            Ok(false)
        }
        Some(false) => {
            runner.execute_tasks();
            Ok(true)
        }
        _ => Err(UserFacingError::TaskExecutionError(ErrWithMessage {
            code: "INVALID_DRY_RUN".to_string(),
            messages: vec!["Invalid dry run value".to_string()],
        })),
    };
}

fn main() {
    let initial_arg_matches = CliArgs::command().get_matches();
    match run_from_matches(initial_arg_matches) {
        Ok(_) => println!("Task execution complete"),
        Err(e) => handle_user_facing_error(e),
    };
}
