mod file_parsing;
mod runners;
mod tests;
mod utils;

use crate::file_parsing::taskfile::Taskfile;
use clap::{value_parser, ArgMatches, CommandFactory, Parser};
use runners::builder::TaskBuilder;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help(true), trailing_var_arg=true )]
struct CliArgs {
    #[arg(default_value = "Taskfile", short, long, help="file path to load tasks from", value_parser=value_parser!(PathBuf))]
    config: PathBuf,

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
fn print_error() -> Result<(), ()> {
    println!("Error: No Valid Taskfile found");
    let mut cmd = CliArgs::command();
    println!("yeet: \n\n\n\n\n\n");
    let help = cmd.render_help().to_string();
    if !cfg!(test) {
        println!("{}", help);
    }
    return Err(());
}
fn run_from_matches(initial_arg_matches: ArgMatches) -> Result<(), ()> {
    let config_path = match initial_arg_matches.get_one::<PathBuf>("config") {
        Some(fp) if fp.exists() => fp.to_str().unwrap().to_string(),
        Some(_) => return print_error(),
        None => return print_error(),
    };
    let config = Taskfile::new(config_path).unwrap();
    // clap will catch any missing or bad args

    let mut builder = TaskBuilder::new(config);
    let dry_run = initial_arg_matches.get_one::<bool>("dry_run");
    let runner = builder.create_task_runner(initial_arg_matches.to_owned());
    match dry_run {
        Some(true) => {
            runner.print_commands();
        }
        Some(false) => {
            runner.execute_tasks();
        }
        _ => (),
    }
    return Ok(());
}
fn main() {
    let initial_arg_matches = CliArgs::command().get_matches();
    let _ = run_from_matches(initial_arg_matches);
}
