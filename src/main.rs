mod config;

use std::{path::PathBuf, process::exit};

use clap::{Parser, CommandFactory, value_parser};
use config::Config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help(true), trailing_var_arg=true )]
struct Args {
    #[arg(default_value = "Taskfile", short, long, help="file path to load tasks from", value_parser=value_parser!(PathBuf))]
    config: PathBuf,

    // All trailing args are captured in vec to be parsed later
    #[arg(trailing_var_arg=true, allow_hyphen_values=true, default_value="--help", help="commands defined by Taskfile")]
    task_info: Vec<String>,
}

fn main() {
    let initial_matches = Args::command().get_matches();
    let config_path = match initial_matches.get_one::<PathBuf>("config") {
        Some(fp) => {
            if fp.exists() {
                fp.to_str().unwrap().to_string()
            }
            else {
            println!("Error: No Taskfile found");
            Args::command().print_help();
            exit(1)
            }
        },
        None => {
            println!("Error: something went wrong");
            Args::command().print_help();
            exit(1)
        },
    };
    let config = Config::new(config_path).unwrap();
    let cmd = config.create_clap_command();

    let raw_args: Vec<_> = initial_matches.get_many::<String>("task_info").unwrap().collect();
    let inputs = cmd.get_matches_from(raw_args);

    // we can be confident in unwraps since CLAP can handle most errors on read above
    let subcmd = inputs.subcommand_name().unwrap();
    let chosen_command = config.get_task_by_name(subcmd).unwrap();
    let (_, subcmd_struct) = inputs.subcommand().unwrap();
    let subcmd_inputs = subcmd_struct.to_owned();
    let _ = chosen_command.stream_command(subcmd_inputs);
}
