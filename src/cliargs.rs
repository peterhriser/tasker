use clap::{value_parser, Parser};
use std::path::PathBuf;
#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help(true), trailing_var_arg=true )]
pub struct CliArgs {
    #[arg(default_value = "Taskfile", short, long="config", help="path to file with task definitions", value_parser=value_parser!(PathBuf))]
    pub config_path: PathBuf,
    #[arg(default_value = "~/.tasker/Taskfile", env, long="global-config", help="global tasker definition", value_parser=value_parser!(PathBuf))]
    pub global_config_path: Option<PathBuf>,

    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        default_value = "help",
        help = "commands defined by Taskfile"
    )]
    pub task_info: Vec<String>,

    #[arg(
        short = 'x',
        long = "context",
        help = "execution context to load for command"
    )]
    pub context: Option<String>,
    #[arg(
        short,
        long,
        help = "print out the commands that would be run instead of executing them"
    )]
    pub dry_run: bool,
}
