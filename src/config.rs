use clap::ArgMatches;
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
    fn get_clap_arg(&self) -> clap::Arg {
        let name_owned = self.name.to_owned();
        if !self.is_required() {
            let default = self.default.to_owned().unwrap();
            return clap::Arg::new(name_owned).default_value(default);
        } else {
            return clap::Arg::new(name_owned).required(true);
        }
    }
}

// task file command is a single defined command stanza from a config
#[derive(Debug, Deserialize)]
pub struct TaskStanza {
    #[serde(rename(deserialize = "cmd"))]
    pub command_template: String,
    #[serde(rename(deserialize = "args"))]
    pub command_args: Vec<CmdArg>,
    pub description: Option<String>,
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

    fn create_clap_subcommand(&self, name: String) -> clap::Command {
        let mut arg_vector: Vec<clap::Arg> = vec![];
        for arg in &self.command_args {
            let new_arg = arg.get_clap_arg();
            arg_vector.push(new_arg)
        }
        let about = self.description.to_owned().unwrap_or_default();
        let base_command = clap::Command::new(name)
            .about(about)
            .arg_required_else_help(true)
            .args(arg_vector);
        return base_command;
    }

    pub fn create_command_string(&self, clap_inputs: ArgMatches) -> String {
        let mut cmd_raw = self.command_template.to_string();
        let replacer_map = self.create_arg_replace_hashmap();
        for arg in &self.command_args {
            let replace_string = replacer_map.get(&arg.name).unwrap();
            let input_string: String = clap_inputs
                .get_one::<String>(&arg.name)
                .unwrap()
                .to_string();
            cmd_raw = cmd_raw.replace(replace_string, &input_string);
        }
        return cmd_raw;
    }

    fn parse_command_from_string(&self, command_str: String) -> Result<Command, io::Error> {
        let mut parts = command_str.split_whitespace();
        let command_name = parts.next().expect("no command specified");
        let args = parts;

        let mut cmd = Command::new(command_name);
        cmd.args(args);
        Ok(cmd)
    }

    fn generate_command_with_args(&self, inputs: ArgMatches) -> Result<Command, io::Error> {
        let cmd_string = self.create_command_string(inputs);
        self.parse_command_from_string(cmd_string)
    }

    fn call_command(&self, clap_inputs: ArgMatches) -> Result<ChildStdout, io::Error> {
        let result = self
            .generate_command_with_args(clap_inputs)
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
    pub fn stream_command(&self, clap_inputs: ArgMatches) -> Result<(), io::Error> {
        let cmd_results = self.call_command(clap_inputs).unwrap();
        let reader = io::BufReader::new(cmd_results);

        io::BufRead::lines(reader).for_each(|line| {
            io::Write::flush(&mut io::stdout()).unwrap();
            println!("{}", line.unwrap())
        });

        println!("Completed Task!");
        Ok(())
    }
}

// Config File made from assembling above structs
#[derive(Deserialize)]
pub(crate) struct Config {
    // TODO: use these when filling in vars
    #[serde(rename(deserialize = "globals"))]
    pub _globals: HashMap<String, AllowedVarTypes>,
    pub commands: HashMap<String, TaskStanza>,
}
impl Config {
    pub fn get_task_by_name(&self, name: &str) -> Option<&TaskStanza> {
        return self.commands.get(name);
    }
    pub fn new(file_path: String) -> Result<Config, std::io::Error> {
        let file = std::fs::File::open(file_path).expect("Could not open file");
        let deserialized_config: Config =
            serde_yaml::from_reader(file).expect("Could not read values.");
        Ok(deserialized_config)
    }
    pub fn create_clap_command(&self) -> clap::Command {
        let mut task_vector: Vec<clap::Command> = vec![];
        for (name, task) in &self.commands {
            let new_command = task.create_clap_subcommand(name.to_string());
            task_vector.push(new_command);
        }
        let base_command = clap::Command::new("tasker")
            .about("about")
            .arg_required_else_help(true)
            .subcommands(task_vector);
        return base_command;
    }
}
