use serde::Deserialize;
use std::collections::HashMap;

use super::taskstanza::TaskStanza;

type TaskContext = HashMap<String, String>;
// Taskfile File made from assembling above structs
#[derive(Deserialize, Clone)]
pub struct Taskfile {
    pub contexts: HashMap<String, TaskContext>,
    pub commands: HashMap<String, TaskStanza>,
}

impl Taskfile {
    pub fn get_task_by_name(&self, name: &str) -> Option<&TaskStanza> {
        return self.commands.get(name);
    }

    pub fn new(file_path: String) -> Result<Taskfile, std::io::Error> {
        let file = std::fs::File::open(file_path).unwrap();
        let base_deserialized_config: Taskfile =
            serde_yaml::from_reader(file).expect("Could not read values.");

        let mut new_deserialized_config = base_deserialized_config.clone();
        // // let selected_context = &base_deserialized_config.get_context(context);
        // for (name, item) in &base_deserialized_config.commands {
        //     let mut new_task_stanza = item.clone();
        //     let mut new_command_args: Vec<CmdArg> = vec![];
        //     for arg in &item.command_args {
        //         let new_default = selected_context.get(&arg.name.to_string());
        //         let new_default_parsed = match new_default {
        //             Some(item) => Some(item.to_owned()),
        //             None => None,
        //         };
        //         let mut new_arg = arg.clone();
        //         new_arg.set_default_from_option(new_default_parsed).unwrap();
        //         new_command_args.push(new_arg);
        //     }
        //     new_task_stanza.set_args(new_command_args);
        //     new_deserialized_config.set_command(name.to_string(), new_task_stanza);
        // }
        Ok(new_deserialized_config)
    }
    pub fn get_context(&self, value: Option<&String>) -> HashMap<String, String> {
        let default = HashMap::<String, String>::new();
        return match value {
            Some(context_name) => {
                let task_context = self.contexts.get(context_name);
                match task_context {
                    Some(context) => context.to_owned(),
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
            .about("tasker runs tasks defined in a Taskfile")
            .color(clap::ColorChoice::Always)
            .no_binary_name(true)
            .arg_required_else_help(true)
            .allow_missing_positional(true)
            .subcommands(task_vector);
        return base_command;
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::load_from_string;

    use super::Taskfile;
    #[test]
    fn test_load_from_yaml() {
        let _ = load_from_string();
    }
    #[test]
    fn test_load_from_file() {
        let _ = Taskfile::new("Taskfile".to_string());
    }
}
// #[test]
// fn test_get_context() {
//     let taskfile = load_from_string();
//     let context_name = "staging".to_string();
//     let context = taskfile.get_context(Some(&context_name));
//     assert_eq!(context.get("name").unwrap().to_owned(), "Peter".to_string());
// }
// #[test]
// fn test_create_clap_command() {
//     let taskfile = load_from_string();
//     let cmd = taskfile.create_clap_command();
//     assert_eq!(
//         cmd.get_about().unwrap().to_string(),
//         "tasker runs tasks defined in a Taskfile"
//     );
//     let arg_count = cmd.get_subcommands().count();
//     assert_eq!(arg_count, 2);
// }
