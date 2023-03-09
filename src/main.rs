mod config;
use clap::{value_parser, CommandFactory, Parser};
use config::Config;
use std::{path::PathBuf, process::exit, result};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help(true), trailing_var_arg=true )]
struct Args {
    #[arg(default_value = "Taskfile", short, long, help="file path to load tasks from", value_parser=value_parser!(PathBuf))]
    config: PathBuf,

    // All trailing args are captured in vec to be parsed later
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
}

fn main() {
    let initial_arg_matches = Args::command().get_matches();
    let config_path = match initial_arg_matches.get_one::<PathBuf>("config") {
        Some(fp) if fp.exists() => {
            fp.to_str().unwrap().to_string()
        }
        Some(_) => {
                println!("Error: No Taskfile found");
                Args::command().print_help().unwrap();
                exit(1)
        }
        None => {
            println!("Error: Not a valid filepath for Taskfile");
            Args::command().print_help().unwrap();
            exit(1)
        }
    };
    // clap will catch any missing or bad args
    let task_context_name = initial_arg_matches.get_one::<String>("context");
    let config = Config::new(config_path, task_context_name).unwrap();
    let command_to_run = config.create_clap_command();

    // we can be confident in unwraps since we verify most values above on load
    let raw_args: Vec<_> = initial_arg_matches
        .get_many::<String>("task_info")
        .unwrap()
        .collect();

    // get matches found so far and parse into subcommand
    let cli_inputs = command_to_run.get_matches_from(raw_args);
    let (subcommand_name, clap_matched_args) = cli_inputs.subcommand().unwrap();
    let selected_task = config.get_task_by_name(subcommand_name).unwrap();
    let task_context = config.get_context(task_context_name);
    match selected_task.execute_command(clap_matched_args.to_owned(), task_context) {
        Ok(_) => {
            println!("Completed Task!");
        }
        Err(_) => {
            println!("Task Failed")
        }
    };
}
