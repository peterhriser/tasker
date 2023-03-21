use std::{
    collections::HashMap,
    io,
    process::{ChildStdout, Command, Stdio},
};

use clap::ArgMatches;
use serde::{Deserialize, Serialize};

use super::cmd::{ArgError, CmdArg};

// task file command is a single defined command stanza from a config
#[derive(Deserialize, Serialize, Clone)]
pub struct TaskStanza {
    #[serde(rename(deserialize = "cmd"))]
    pub unparsed_command_raw: String,
    #[serde(rename(deserialize = "args"))]
    pub command_args: Vec<CmdArg>,
    pub description: Option<String>,
}

impl TaskStanza {
    pub(super) fn set_args(&mut self, args: Vec<CmdArg>) {
        self.command_args = args;
    }
    fn create_arg_replace_hashmap(&self) -> HashMap<String, String> {
        let mut lookup_map: HashMap<String, String> = HashMap::new();
        for arg in &self.command_args {
            let search_term = format!("${{{arg_name}}}", arg_name = arg.name);
            if self.unparsed_command_raw.contains(&search_term) {
                let found_term = arg.name.to_string();
                lookup_map.insert(found_term, search_term);
            }
        }
        return lookup_map;
    }
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

    pub(super) fn create_command_string(
        &self,
        clap_inputs: ArgMatches,
        context: HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut cmd_raw = self.unparsed_command_raw.to_string();
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

#[cfg(test)]
pub mod taskstanza_test_helpers {
    use super::TaskStanza;
    use crate::config::cmd::cmd_test_helpers::create_cmd_arg_for_test;

    pub fn create_task_stanza_for_tests(optional_arg: bool) -> TaskStanza {
        if optional_arg {
            return TaskStanza {
                unparsed_command_raw: "echo ${required_arg} ${optional_arg}".to_string(),
                command_args: vec![
                    create_cmd_arg_for_test(true),
                    create_cmd_arg_for_test(false),
                ],
                description: Some("this has a required and optional arg".to_string()),
            };
        } else {
            return TaskStanza {
                unparsed_command_raw: "echo ${required_arg}".to_string(),
                command_args: vec![
                    create_cmd_arg_for_test(true),
                    create_cmd_arg_for_test(false),
                ],
                description: Some("this has a required and optional arg".to_string()),
            };
        }
    }
}
#[cfg(test)]
mod tests {
    use super::taskstanza_test_helpers::create_task_stanza_for_tests;
    use std::collections::HashMap;

    #[test]
    fn test_create_hashmap_of_args() {
        let stanza = create_task_stanza_for_tests(true);
        let map = stanza.create_arg_replace_hashmap();
        let expected_map: HashMap<String, String> = HashMap::from([
            ("required_arg".to_string(), "${required_arg}".to_string()),
            ("optional_arg".to_string(), "${optional_arg}".to_string()),
        ]);
        for key in expected_map.keys() {
            assert_eq!(map.get(key), expected_map.get(key))
        }
    }
    #[test]
    fn test_create_hashmap_extra_args() {
        let stanza = create_task_stanza_for_tests(false);
        let map = stanza.create_arg_replace_hashmap();
        let expected_map: HashMap<String, String> = HashMap::from([
            ("required_arg".to_string(), "${required_arg}".to_string()),
        ]);
        for key in expected_map.keys() {
            assert_eq!(map.get(key), expected_map.get(key))
        }
    }
}