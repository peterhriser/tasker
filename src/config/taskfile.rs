use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::cmd::CmdArg;
use super::taskcontext::TaskContext;
use super::taskstanza::TaskStanza;

// Taskfile File made from assembling above structs
#[derive(Deserialize, Serialize, Clone)]
pub struct Taskfile {
    pub contexts: HashMap<String, TaskContext>,
    pub commands: HashMap<String, TaskStanza>,
}

impl Taskfile {
    pub fn get_task_by_name(&self, name: &str) -> Option<&TaskStanza> {
        return self.commands.get(name);
    }

    pub fn new(file_path: String, context: Option<&String>) -> Result<Taskfile, std::io::Error> {
        let file = std::fs::File::open(file_path).unwrap();
        let base_deserialized_config: Taskfile =
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
                new_arg.set_default_from_option(new_default_parsed).unwrap();
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
