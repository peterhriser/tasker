use std::fmt;

use super::cmd::CmdArg;
use serde::{de::Visitor, Deserialize, Deserializer};

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]

pub enum UnparsedCommandEnum {
    Task(String),
    Shell(String),
    Script(String),
}
#[derive(Clone, Debug)]
pub struct TaskCmd {
    pub key: UnparsedCommandEnum,
    pub value: String,
}
impl<'de> Deserialize<'de> for TaskCmd {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TaskCmdVisitor;

        impl<'de> Visitor<'de> for TaskCmdVisitor {
            type Value = TaskCmd;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a YAML object with enum key-value pairs")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut key = None;
                let mut value = None;

                while let Some(k) = map.next_key::<String>()? {
                    let v = map.next_value::<String>()?;

                    if key.is_some() {
                        return Err(serde::de::Error::duplicate_field("duplicate key"));
                    }

                    key = Some(k);
                    value = Some(v);
                }

                let key = key.ok_or_else(|| serde::de::Error::missing_field("key"))?;
                let value = value.ok_or_else(|| serde::de::Error::missing_field("value"))?;

                let key = match key.as_str() {
                    "task" => UnparsedCommandEnum::Task("task".to_string()),
                    "shell" => UnparsedCommandEnum::Shell("shell".to_string()),
                    "script" => UnparsedCommandEnum::Script("script".to_string()),
                    _ => {
                        return Err(serde::de::Error::unknown_field(
                            &key,
                            &["task", "shell", "script"],
                        ))
                    }
                };

                Ok(TaskCmd { key, value })
            }
        }

        deserializer.deserialize_map(TaskCmdVisitor)
    }
}

impl fmt::Display for UnparsedCommandEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match &self {
            UnparsedCommandEnum::Task(value) => write!(f, "{}", value),
            UnparsedCommandEnum::Shell(value) => write!(f, "{}", value),
            UnparsedCommandEnum::Script(value) => write!(f, "{}", value),
        };
    }
}

// task file command is a single defined command stanza from a config
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TaskStanza {
    pub name: String,
    pub commands: Vec<TaskCmd>,
    #[serde(rename(deserialize = "args"))]
    pub command_args: Vec<CmdArg>,
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
}

#[cfg(test)]
mod tests {
    use super::{TaskCmd, TaskStanza};
    use crate::config::{cmd::CmdArg, taskstanza::UnparsedCommandEnum};

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
                key: UnparsedCommandEnum::Task("test".to_string()),
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
