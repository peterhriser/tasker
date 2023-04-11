use std::fmt;

use serde::{de::Visitor, Deserialize, Deserializer};

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CommandTypes {
    Task(String),
    Shell(String),
    Script(String),
    Cwd(String),
}
impl fmt::Display for CommandTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match &self {
            CommandTypes::Task(value) => write!(f, "{}", value),
            CommandTypes::Shell(value) => write!(f, "{}", value),
            CommandTypes::Script(value) => write!(f, "{}", value),
            CommandTypes::Cwd(value) => write!(f, "{}", value),
        };
    }
}

#[derive(Clone, Debug)]
pub struct TaskCmd {
    pub key: CommandTypes,
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
                    "task" => CommandTypes::Task("task".to_string()),
                    "shell" => CommandTypes::Shell("shell".to_string()),
                    "script" => CommandTypes::Script("script".to_string()),
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

// cmd arg stanzas
#[derive(Deserialize, Clone)]
pub struct CmdArg {
    pub name: String,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(rename = "type")]
    pub arg_type: String,
}
impl CmdArg {
    fn is_required(&self) -> bool {
        if self.default.is_none() {
            return true;
        }
        return false;
    }
    pub(super) fn get_clap_arg(&self) -> clap::Arg {
        let name_owned = self.name.to_owned();
        if !self.is_required() {
            let default = self.default.to_owned().unwrap();
            return clap::Arg::new(name_owned).default_value(default);
        } else {
            return clap::Arg::new(name_owned).required(true);
        }
    }
    pub fn get_name(&self) -> &str {
        return &self.name;
    }
    pub fn get_default(&self) -> Option<&str> {
        return self.default.as_deref();
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_cmd_arg() {
        let arg = super::CmdArg {
            name: "test".to_string(),
            default: Some("default".to_string()),
            arg_type: "string".to_string(),
        };
        let clap_arg = arg.get_clap_arg();
        assert_eq!(clap_arg.get_id(), "test");
        assert_eq!(clap_arg.is_required_set(), false);
        assert_eq!(clap_arg.get_default_values(), &["default"]);
    }

    #[test]
    fn test_display_command_types() {
        let task = super::CommandTypes::Task("task".to_string());
        let shell = super::CommandTypes::Shell("shell".to_string());
        let script = super::CommandTypes::Script("script".to_string());
        assert_eq!(task.to_string(), "task");
        assert_eq!(shell.to_string(), "shell");
        assert_eq!(script.to_string(), "script");
    }
    #[test]
    fn test_deserialize_task_cmd() {
        let yaml = r#"
        task: "test"
        "#;
        let task_cmd: super::TaskCmd = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(task_cmd.key.to_string(), "task");
        assert_eq!(task_cmd.value, "test");
    }
    #[test]
    fn test_deserialize_task_cmd_fmt_error() {
        let yaml = r#"
        task: "test"
        task: "test"
        "#;
        let task_cmd: Result<super::TaskCmd, _> = serde_yaml::from_str(yaml);
        assert_eq!(
            task_cmd.unwrap_err().to_string(),
            "duplicate field `duplicate key` at line 2 column 9"
        );
    }

    #[test]
    fn test_cmd_arg_get_name() {
        let arg = super::CmdArg {
            name: "test".to_string(),
            default: Some("default".to_string()),
            arg_type: "string".to_string(),
        };
        assert_eq!(arg.get_name(), "test");
    }
    #[test]
    fn test_cmd_arg_get_default() {
        let arg = super::CmdArg {
            name: "test".to_string(),
            default: Some("default".to_string()),
            arg_type: "string".to_string(),
        };
        assert_eq!(arg.get_default(), Some("default"));
    }
    #[test]
    fn test_deserialize_task_cmd_unknown_key() {
        let yaml = r#"
        test: "test"
        "#;
        let task_cmd: Result<super::TaskCmd, _> = serde_yaml::from_str(yaml);
        assert_eq!(
            task_cmd.unwrap_err().to_string(),
            "unknown field `test`, expected one of `task`, `shell`, `script` at line 2 column 9"
        );
    }
}
