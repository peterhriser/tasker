use super::cmd::{CmdArg, TaskCmd};
use serde::Deserialize;

// task file command is a single defined command stanza from a config
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TaskStanza {
    pub name: String,
    pub commands: Vec<TaskCmd>,
    #[serde(rename(deserialize = "args"))]
    command_args: Vec<CmdArg>,
    pub description: Option<String>,
}

impl TaskStanza {
    pub(super) fn create_clap_subcommand(&self) -> clap::Command {
        let mut arg_vector: Vec<clap::Arg> = vec![];
        for arg in &self.command_args {
            let new_arg = arg.get_clap_arg();
            arg_vector.push(new_arg)
        }
        let about = self.description.to_owned().unwrap_or_default();
        let base_command = clap::Command::new(&self.name).about(about).args(arg_vector);
        return base_command;
    }

    pub fn get_command_args(&self) -> &Vec<CmdArg> {
        &self.command_args
    }
}

#[cfg(test)]
mod tests {
    use super::{TaskCmd, TaskStanza};
    use crate::taskfile::cmd::{CmdArg, CommandTypes};

    #[test]
    fn test_create_clap_subcommand() {
        let mut arg_vector: Vec<CmdArg> = vec![];
        let arg1 = CmdArg {
            name: "arg1".to_string(),
            default: Some("default".to_string()),
            arg_type: "string".to_string(),
        };
        let arg2 = CmdArg {
            name: "arg2".to_string(),
            default: None,
            arg_type: "string".to_string(),
        };
        arg_vector.push(arg1);
        arg_vector.push(arg2);
        let task_stanza = TaskStanza {
            name: "test".to_string(),
            commands: vec![TaskCmd {
                key: CommandTypes::Task("test".to_string()),
                value: "test".to_string(),
            }],
            command_args: arg_vector,
            description: None,
        };
        let subcommand = task_stanza.create_clap_subcommand();
        let mut args = subcommand.get_arguments();
        assert_eq!(subcommand.get_name(), "test");
        assert_eq!(subcommand.get_about().unwrap().to_string(), "");
        let arg_1 = args.next().unwrap();
        assert_eq!(arg_1.get_id(), "arg1");
        assert_eq!(arg_1.get_default_values(), &["default"]);
        let arg_2 = args.next().unwrap();
        assert_eq!(arg_2.get_id(), "arg2");
        assert_eq!(arg_2.get_default_values().is_empty(), true);
    }
}
