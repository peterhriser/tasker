use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::{fmt, io};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllowedVarTypes {
    U(u64),
    S(String),
    V(Vec<AllowedVarTypes>),
}

impl fmt::Display for AllowedVarTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AllowedVarTypes::U(u) => write!(f, "{}", u),
            AllowedVarTypes::S(s) => write!(f, "{}", s),
            AllowedVarTypes::V(v) => {
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CmdArg {
    pub name: String,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub choices: Option<Vec<String>>,
}
impl CmdArg {
    pub fn is_input_usable(&self, input: String) -> bool {
        match &self.choices {
            Some(choices) => return choices.contains(&input),
            None => return true,
        }
    }
    fn is_required(&self) -> bool {
        if self.default.is_none() {
            return true;
        }
        return false;
    }
    fn get_default(&self) -> String {
        match &self.default {
            Some(def) => return def.to_owned(),
            None => {
                panic!("Attempting to access non-existent default")
            }
        };
    }
}
#[derive(Debug, Deserialize)]
pub struct Cmd {
    pub raw_command: String,
}

fn cmd_deserialize<'de, D>(deserializer: D) -> Result<Cmd, D::Error>
where
    D: Deserializer<'de>,
{
    let str_sequence = String::deserialize(deserializer)?;
    return Ok(Cmd {
        raw_command: str_sequence,
    });
}

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub globals: HashMap<String, AllowedVarTypes>,
    pub commands: HashMap<String, TaskfileCommand>,
}
impl Config {
    pub fn get_task_from_name(&self, name: &str) -> Option<&TaskfileCommand> {
        return self.commands.get(name);
    }
}

#[derive(Debug, Deserialize)]
pub struct TaskfileCommand {
    #[serde(deserialize_with = "cmd_deserialize")]
    pub cmd: Cmd,
    pub args: Vec<CmdArg>,
}

impl TaskfileCommand {
    fn create_arg_replace_hashmap(&self) -> HashMap<String, String> {
        let mut lookup_map: HashMap<String, String> = HashMap::new();
        for arg in &self.args {
            let search_term = format!("${{{arg_name}}}", arg_name = arg.name);
            if self.cmd.raw_command.contains(&search_term) {
                let found_term = arg.name.to_string();
                lookup_map.insert(found_term, search_term);
            }
        }
        return lookup_map;
    }
}

// tie the parsed yaml and the input command togehter
pub struct StagedTask<'a> {
    pub(crate) selected_command: &'a TaskfileCommand,
    pub(crate) command_inputs: Vec<String>,
}
impl StagedTask<'_> {
    pub fn create_command_string(&self) -> String {
        let mut cmd_raw = self.selected_command.cmd.raw_command.to_string();
        let replacer_map = self.selected_command.create_arg_replace_hashmap();
        let identified_inputs = self.match_inputs_to_args();
        for (name, value) in identified_inputs {
            let replace_string = replacer_map.get(&name).unwrap();
            cmd_raw = cmd_raw.replace(replace_string, &value);
        }
        return cmd_raw;
    }
    fn match_inputs_to_args(&self) -> HashMap<String, String> {
        let mut parameter_to_value_map: HashMap<String, String> = HashMap::new();
        let max_len = self.command_inputs.len();
        for i in 0..max_len {
            let potential_arg = &self.selected_command.args[i];
            let potential_input = &self.command_inputs[i];
            if potential_arg.is_input_usable(potential_input.to_string()) {
                parameter_to_value_map.insert(
                    potential_arg.name.to_string(),
                    self.command_inputs[i].to_string(),
                );
            }
        }
        let remaining_args = &self.selected_command.args[max_len..];
        for arg in remaining_args {
            if arg.is_required() {
                panic!("Missing {}", arg.name);
            } else {
                parameter_to_value_map.insert(arg.name.to_string(), arg.get_default());
            }
        }
        return parameter_to_value_map;
    }
    pub fn parse_command_from_string(&self, command_str: String) -> Result<Command, io::Error> {
        let mut parts = command_str.split_whitespace();
        let command_name = parts.next().expect("no command specified");
        let args = parts;

        let mut cmd = Command::new(command_name);
        cmd.args(args);
        Ok(cmd)
    }

    pub fn generate_command_with_args(&self) -> Result<Command, io::Error> {
        let cmd_string = self.create_command_string();
        self.parse_command_from_string(cmd_string)
    }
}
