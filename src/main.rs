mod config;
mod runners;
mod utils;

use crate::config::taskfile::Taskfile;
use clap::{value_parser, ArgMatches, CommandFactory, Parser};
use runners::runner::TaskRunner;
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
}

fn run_from_matches(initial_arg_matches: ArgMatches) -> Result<(), ()> {
    let config_path = match initial_arg_matches.get_one::<PathBuf>("config") {
        Some(fp) if fp.exists() => fp.to_str().unwrap().to_string(),
        Some(_) => {
            println!("Error: No Taskfile found");
            if !cfg!(test) {
                CliArgs::command().print_help().unwrap();
            }
            return Err(());
        }
        None => {
            println!("Error: Not a valid filepath for Taskfile");
            if !cfg!(test) {
                CliArgs::command().print_help().unwrap();
            }
            return Err(());
        }
    };
    let config = Taskfile::new(config_path).unwrap();
    // clap will catch any missing or bad args

    let mut runner = TaskRunner::new(config);
    runner.execute_task(initial_arg_matches);
    return Ok(());
}
fn main() {
    let initial_arg_matches = CliArgs::command().get_matches();
    let _ = run_from_matches(initial_arg_matches);
}

#[cfg(test)]
pub mod test_helpers {
    use crate::config::taskfile::Taskfile;
    pub fn load_from_string() -> Taskfile {
        let example_file = r#"project: "Example"
version: "1.0"
author: "Peter"
contexts:
  test:
    test_key: test_value
tasks:
  - name: greet
    commands:
    - shell: echo Hello ${first_name} ${last_name}
    description: "greets a user by name"
    args:
      - name: first_name
        type: string
      - name: last_name
        type: string
        default: "the First"
"#;
        return serde_yaml::from_str(example_file).unwrap();
    }
}
#[cfg(test)]
mod tests {
    use crate::{run_from_matches, CliArgs};
    use clap::CommandFactory;

    #[test]
    fn test_entry_point() {
        let initial_arg_matches =
            CliArgs::command().get_matches_from(vec!["tasker", "hello", "Peter"]);
        let result = run_from_matches(initial_arg_matches);
        assert!(result.is_ok())
    }
    #[test]
    fn test_missing_file() {
        let initial_arg_matches =
            CliArgs::command().get_matches_from(vec!["tasker", "-c", "fakefile", "hello", "Peter"]);
        let result = run_from_matches(initial_arg_matches);
        assert!(result.is_err())
    }
}
