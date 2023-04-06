use serde::Deserialize;
use std::collections::HashMap;

use super::taskstanza::TaskStanza;

type TaskContext = HashMap<String, String>;
// Taskfile File made from assembling above structs
#[derive(Deserialize, Clone)]
pub struct Taskfile {
    pub contexts: HashMap<String, TaskContext>,
    pub tasks: Vec<TaskStanza>,
}

impl Taskfile {
    pub fn new(file_path: String) -> Result<Taskfile, std::io::Error> {
        let file = std::fs::File::open(file_path).unwrap();
        let base_deserialized_config: Taskfile = serde_yaml::from_reader(file).unwrap();
        Ok(base_deserialized_config)
    }
    pub fn get_task_by_name(&self, name: &str) -> Option<&TaskStanza> {
        return self.tasks.iter().find(|&obj| obj.name == name);
    }
    pub fn get_context(&self, value: Option<String>) -> HashMap<String, String> {
        let default = HashMap::<String, String>::new();
        return match value {
            Some(context_name) => {
                let task_context = self.contexts.get(&context_name);
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
        for task in &self.tasks {
            let new_command = task.create_clap_subcommand();
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
    use super::Taskfile;
    use crate::utils::test_helpers::test_helpers::load_from_string;

    #[test]
    fn test_load_from_yaml() {
        let _ = load_from_string();
    }
    #[test]
    fn test_load_from_file() {
        let _ = Taskfile::new("Taskfile".to_string());
    }
    #[test]
    fn test_get_task_by_name() {
        let taskfile = load_from_string();
        let task = taskfile.get_task_by_name("test-cmd");
        assert!(task.is_some());
    }
    #[test]
    fn test_get_context() {
        let taskfile = load_from_string();
        let context = taskfile.get_context(Some("test".to_string()));
        assert_eq!(context.get("test_key"), Some(&"test_value".to_string()));
    }
    #[test]
    fn test_get_context_none() {
        let taskfile = load_from_string();
        let context = taskfile.get_context(None);
        assert_eq!(context.get("test"), None);
    }
}
