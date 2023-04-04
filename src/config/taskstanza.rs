use std::fmt;

use serde::Deserialize;

use super::cmd::CmdArg;

#[derive(Clone, Deserialize, Debug)]
pub enum UnparsedCommandEnum {
    #[serde(rename(deserialize = "tasks"))]
    Tasks(String),
    #[serde(rename(deserialize = "cmds"))]
    Cmds(String),
}
impl fmt::Display for UnparsedCommandEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match &self {
            UnparsedCommandEnum::Tasks(value) => write!(f, "{}", value),
            UnparsedCommandEnum::Cmds(value) => write!(f, "{}", value),
        };
    }
}
// task file command is a single defined command stanza from a config
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TaskStanza {
    #[serde(flatten)]
    pub unparsed_commands: UnparsedCommandEnum,
    #[serde(rename(deserialize = "args"))]
    pub command_args: Vec<CmdArg>,
    pub description: Option<String>,
}

impl TaskStanza {
    pub(super) fn create_clap_subcommand(&self, name: String) -> clap::Command {
        let mut arg_vector: Vec<clap::Arg> = vec![];
        for arg in &self.command_args {
            let new_arg = arg.get_clap_arg();
            arg_vector.push(new_arg)
        }
        let about = self.description.to_owned().unwrap_or_default();
        let base_command = clap::Command::new(name).about(about).args(arg_vector);
        return base_command;
    }
    pub fn is_subtask(&self) -> bool {
        match self.unparsed_commands {
            UnparsedCommandEnum::Cmds(_) => false,
            UnparsedCommandEnum::Tasks(_) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TaskStanza;
    use crate::config::cmd::CmdArg;
    use crate::config::taskstanza::UnparsedCommandEnum;

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
            unparsed_commands: UnparsedCommandEnum::Tasks("test".to_string()),
            command_args: arg_vector,
            description: None,
        };
        let subcommand = task_stanza.create_clap_subcommand("test".to_string());
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
    #[test]
    fn test_is_subtask() {
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
            unparsed_commands: UnparsedCommandEnum::Tasks("test".to_string()),
            command_args: arg_vector.to_owned(),
            description: None,
        };
        assert_eq!(task_stanza.is_subtask(), true);
        let task_stanza = TaskStanza {
            unparsed_commands: UnparsedCommandEnum::Cmds("test".to_string()),
            command_args: arg_vector.to_owned(),
            description: None,
        };
        assert_eq!(task_stanza.is_subtask(), false);
    }
}
