use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::process::{ChildStdout, Command, Stdio};

#[derive(Debug, Deserialize)]
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

// cmd arg stanzas
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

// task file command is a single defined command stanza from a config
#[derive(Debug, Deserialize)]
pub struct TaskStanza {
    pub command_template: String,
    pub command_args: Vec<CmdArg>,
}

impl TaskStanza {
    fn create_arg_replace_hashmap(&self) -> HashMap<String, String> {
        let mut lookup_map: HashMap<String, String> = HashMap::new();
        for arg in &self.command_args {
            let search_term = format!("${{{arg_name}}}", arg_name = arg.name);
            if self.command_template.contains(&search_term) {
                let found_term = arg.name.to_string();
                lookup_map.insert(found_term, search_term);
            }
        }
        return lookup_map;
    }
}

// Config File made from assembling above structs
#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    // TODO: use these when filling in vars
    pub globals: HashMap<String, AllowedVarTypes>,
    pub commands: HashMap<String, TaskStanza>,
}
impl Config {
    pub fn get_task_from_name(&self, name: &str) -> Option<&TaskStanza> {
        return self.commands.get(name);
    }
}

// Staged Task is tying the task stanza to arguments input by users
pub struct StagedTask<'a> {
    pub(crate) selected_command: &'a TaskStanza,
    pub(crate) command_inputs: Vec<String>,
}
impl StagedTask<'_> {
    pub fn create_command_string(&self) -> String {
        let mut cmd_raw = self.selected_command.command_template.to_string();
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
            let potential_arg = &self.selected_command.command_args[i];
            let potential_input = &self.command_inputs[i];
            if potential_arg.is_input_usable(potential_input.to_string()) {
                parameter_to_value_map.insert(
                    potential_arg.name.to_string(),
                    self.command_inputs[i].to_string(),
                );
            }
        }
        let remaining_args = &self.selected_command.command_args[max_len..];
        for arg in remaining_args {
            if arg.is_required() {
                panic!("Missing {}", arg.name);
            } else {
                parameter_to_value_map.insert(arg.name.to_string(), arg.get_default());
            }
        }
        return parameter_to_value_map;
    }
    fn parse_command_from_string(&self, command_str: String) -> Result<Command, io::Error> {
        let mut parts = command_str.split_whitespace();
        let command_name = parts.next().expect("no command specified");
        let args = parts;

        let mut cmd = Command::new(command_name);
        cmd.args(args);
        Ok(cmd)
    }

    fn generate_command_with_args(&self) -> Result<Command, io::Error> {
        let cmd_string = self.create_command_string();
        self.parse_command_from_string(cmd_string)
    }

    fn call_command(&self) -> Result<ChildStdout, io::Error> {
        let result = self
            .generate_command_with_args()
            .unwrap()
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .stdout
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, "Could not capture standard output.")
            })?;

        Ok(result)
    }
    pub fn stream_command(&self) -> Result<(), io::Error> {
        let cmd_results = self.call_command().unwrap();
        let reader = io::BufReader::new(cmd_results);

        io::BufRead::lines(reader).for_each(|line| {
            io::Write::flush(&mut io::stdout()).unwrap();
            println!("{}", line.unwrap())
        });

        println!("Completed Task!");
        Ok(())
    }
}
