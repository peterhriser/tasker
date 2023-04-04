mod config;
mod runners;

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
    runner.setup_task_environment(initial_arg_matches);
    return Ok(());
}
fn main() {
    let initial_arg_matches = CliArgs::command().get_matches();
    let _ = run_from_matches(initial_arg_matches);
}

#[cfg(test)]
pub mod test_helpers {
    use crate::config::{
        cmd::CmdArg,
        taskfile::Taskfile,
        taskstanza::{TaskStanza, UnparsedCommandEnum},
    };

    pub fn create_cmd_arg_for_test(required: bool) -> CmdArg {
        if required {
            return CmdArg {
                name: "required_arg".to_string(),
                default: None,
                arg_type: "string".to_string(),
            };
        } else {
            return CmdArg {
                name: "optional_arg".to_string(),
                default: Some("DefaultValue".to_string()),
                arg_type: "string".to_string(),
            };
        }
    }
    pub fn create_task_stanza_for_tests(optional_arg: bool) -> TaskStanza {
        if optional_arg {
            return TaskStanza {
                unparsed_commands: UnparsedCommandEnum::Cmds(
                    "echo ${required_arg} ${optional_arg}".to_string(),
                ),
                command_args: vec![
                    create_cmd_arg_for_test(true),
                    create_cmd_arg_for_test(false),
                ],
                description: Some("this has a required and optional arg".to_string()),
            };
        } else {
            return TaskStanza {
                unparsed_commands: UnparsedCommandEnum::Cmds("echo ${required_arg}".to_string()),
                command_args: vec![
                    create_cmd_arg_for_test(true),
                    create_cmd_arg_for_test(false),
                ],
                description: Some("this has a required only".to_string()),
            };
        }
    }
    pub fn load_from_string() -> Taskfile {
        let example_file = r#"project: "Example"
version: "1.0"
author: "Peter"
contexts:
  staging:
    vars:
      name: Peter
      last_name: Riser
  prod:
    vars:
      name: Peter "Lord DevOp"
commands:
  hello:
    cmds: echo ${name} ${last_name}
    description: "greets a user"
    args:
      - name: name
        type: string
      - name: last_name
        type: string
        default: "the First"
  tail-log:
    cmds: tail -f /var/log/${log_name}
    description: "tails a log in /var/log/"
    args:
      - name: log_name
        type: string
        default: syslog
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
