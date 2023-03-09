use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::process::{ChildStdout, Command, Stdio};

#[derive(Debug)]
struct ArgError {
    message: String,
}

impl std::error::Error for ArgError {}

impl std::fmt::Display for ArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing Value: {}", self.message)
    }
}

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
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    fn set_default_from_option(&mut self, new_default: Option<String>) {
        match new_default {
            Some(item) => {
                let copied_val = item.to_string();
                self.default = Some(copied_val);
            }
            None => {}
        };
    }
}

// task file command is a single defined command stanza from a config
#[derive(Deserialize, Serialize, Clone)]
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

    fn set_args(&mut self, args: Vec<CmdArg>) {
        self.command_args = args;
    }
    fn create_clap_subcommand(&self, name: String) -> clap::Command {
        let mut arg_vector: Vec<clap::Arg> = vec![];
        for arg in &self.command_args {
            let new_arg = arg.get_clap_arg();
            arg_vector.push(new_arg)
        }
        let about = self.description.to_owned().unwrap_or_default();
        let base_command = clap::Command::new(name).about(about).args(arg_vector);
        return base_command;
    }

    pub fn create_command_string(
        &self,
        clap_inputs: ArgMatches,
        context: HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut cmd_raw = self.command_template.to_string();
        let replacer_map = self.create_arg_replace_hashmap();
        for arg in &self.command_args {
            let replace_string = replacer_map.get(&arg.name).unwrap();
            let input_string: String = match clap_inputs.get_one::<String>(&arg.name) {
                Some(val) => val.to_string(),
                None => match context.get(&arg.name) {
                    Some(val) => val.to_string(),
                    None => {
                        let err = ArgError {
                            message: format!("variable string is {}", &arg.name),
                        };
                        return Err(Box::new(err));
                    }
                },
            };
            cmd_raw = cmd_raw.replace(replace_string, &input_string);
        }
        return Ok(cmd_raw);
    }

    fn parse_command_from_string(&self, command_str: String) -> Command {
        let mut parts = command_str.split_whitespace();
        let command_name = parts.next().expect("no command specified");
        let args = parts;

        let mut cmd = Command::new(command_name);
        cmd.args(args);
        cmd
    }

    fn generate_command_with_args(
        &self,
        clap_inputs: ArgMatches,
        context: HashMap<String, String>,
    ) -> Result<Command, Box<dyn std::error::Error>> {
        let cmd_string = self.create_command_string(clap_inputs, context);
        return match cmd_string {
            Ok(cmd_val) => Ok(self.parse_command_from_string(cmd_val)),
            Err(err) => panic!("{:?}", err),
        };
    }

    fn call_command(
        &self,
        clap_inputs: ArgMatches,
        context: HashMap<String, String>,
    ) -> Result<ChildStdout, io::Error> {
        let result = self
            .generate_command_with_args(clap_inputs, context)
            .unwrap()
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .stdout
            .unwrap();

        Ok(result)
    }

    pub fn execute_command(
        &self,
        clap_inputs: ArgMatches,
        context: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cmd_results = match self.call_command(clap_inputs, context) {
            Ok(val) => val,
            Err(err) => panic!("{:?}", err),
        };
        let reader = io::BufReader::new(cmd_results);

        io::BufRead::lines(reader).for_each(|line| {
            io::Write::flush(&mut io::stdout()).unwrap();
            println!("> {}", line.unwrap())
        });

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TaskContext {
    pub vars: HashMap<String, String>,
}

// Config File made from assembling above structs
#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct Config {
    pub contexts: HashMap<String, TaskContext>,
    pub commands: HashMap<String, TaskStanza>,
}
impl Config {
    pub fn get_task_by_name(&self, name: &str) -> Option<&TaskStanza> {
        return self.commands.get(name);
    }

    pub fn new(file_path: String, context: Option<&String>) -> Result<Config, std::io::Error> {
        let file = std::fs::File::open(file_path).unwrap();
        let base_deserialized_config: Config =
            serde_yaml::from_reader(file).expect("Could not read values.");

        let mut new_deserialized_config = base_deserialized_config.clone();
        let selected_context = &base_deserialized_config.get_context(context);
        for (name, item) in &base_deserialized_config.commands {
            let mut new_task_stanza = item.clone();
            let mut new_command_args: Vec<CmdArg> = vec![];
            for arg in &item.command_args {
                let new_default = selected_context.get(&arg.name.to_string());
                let new_default_parsed = match new_default {
                    Some(item) => Some(item.to_owned()),
                    None => None,
                };
                let mut new_arg = arg.clone();
                new_arg.set_default_from_option(new_default_parsed);
                new_command_args.push(new_arg);
            }
            new_task_stanza.set_args(new_command_args);
            new_deserialized_config.set_command(name.to_string(), new_task_stanza);
        }
        Ok(new_deserialized_config)
    }
    fn set_command(&mut self, name: String, command: TaskStanza) {
        self.commands.insert(name, command);
    }
    pub fn get_context(&self, value: Option<&String>) -> HashMap<String, String> {
        let default = HashMap::<String, String>::new();
        return match value {
            Some(context_name) => {
                let task_context = self.contexts.get(context_name);
                match task_context {
                    Some(context) => context.vars.to_owned(),
                    None => default,
                }
            }
            None => default,
        };
    }
    pub fn create_clap_command(&self) -> clap::Command {
        let mut task_vector: Vec<clap::Command> = vec![];
        for (name, task) in &self.commands {
            let new_command = task.create_clap_subcommand(name.to_string());
            task_vector.push(new_command);
        }
        let base_command = clap::Command::new("tasker")
            .about("tasker runs tasks defined by the taskfile defined in root")
            .color(clap::ColorChoice::Always)
            .no_binary_name(true)
            .arg_required_else_help(true)
            .allow_missing_positional(true)
            .subcommands(task_vector);
        return base_command;
    }
}
