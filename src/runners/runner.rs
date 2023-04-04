use std::collections::HashMap;

use clap::ArgMatches;

use crate::config::{
    taskfile::Taskfile,
    taskstanza::{TaskStanza, UnparsedCommandEnum},
};

pub struct TaskRunner {
    config: Taskfile,
    variable_lookup: HashMap<String, String>,
    clap_config: clap::Command,
}

impl TaskRunner {
    pub fn new(config: Taskfile) -> Self {
        let clp_config = config.create_clap_command();
        Self {
            config,
            variable_lookup: HashMap::new(),
            clap_config: clp_config,
        }
    }
    fn get_config(&self) -> &Taskfile {
        &self.config
    }
    fn upsert_into_variable_map(&mut self, key: String, value: String) {
        if let Some(existing_value) = self.variable_lookup.get_mut(&key) {
            *existing_value = value;
        } else {
            self.variable_lookup.insert(key, value);
        }
    }
    // used for getting defaults and subtask values
    fn update_variables_from_task_stanza(&mut self, task: TaskStanza) {
        for cmd in task.command_args {
            let value = match cmd.default {
                Some(value) => value,
                None => continue,
            };
            let key = cmd.name;
            self.upsert_into_variable_map(key, value);
        }
    }
    fn update_variables_from_arg_matches(&mut self, args: &ArgMatches) {
        for id in args.ids() {
            let key = id.to_string();
            let value = args.get_one::<String>(id.as_str()).unwrap().to_string();
            self.upsert_into_variable_map(key, value);
        }
    }
    fn update_variables_from_context(&mut self, context: HashMap<String, String>) {
        for (key, value) in context.iter() {
            self.upsert_into_variable_map(key.to_owned(), value.to_owned());
        }
    }
    fn is_subtask(&self, task: TaskStanza) -> bool {
        match task.unparsed_commands {
            UnparsedCommandEnum::Cmds(_) => false,
            UnparsedCommandEnum::Tasks(_) => true,
        }
    }
    fn get_subtask(&self, task: TaskStanza) -> (String, TaskStanza) {
        match task.unparsed_commands {
            UnparsedCommandEnum::Cmds(_) => panic!("this is not a subtask"),
            UnparsedCommandEnum::Tasks(task_name) => {
                let name = task_name.split(" ").next().unwrap();
                (
                    name.to_string(),
                    self.config.get_task_by_name(name).unwrap().to_owned(),
                )
            }
        }
    }
    fn get_command_string_parsed(&mut self, task: TaskStanza) -> String {
        let command_string = match self.is_subtask(task.to_owned()) {
            true => {
                let raw_cmd = task.unparsed_commands.to_string();
                let (subtask_name, subtask) = self.get_subtask(task);
                let task_args = raw_cmd
                    .split(" ")
                    .collect::<Vec<&str>>()
                    .split_at(1)
                    .1
                    .join(" ");
                let task_args_parsed = self.replace_string_with_args(task_args);
                println!("{} {}", subtask_name.as_str(), task_args_parsed.as_str());
                let complete_args_parsed =
                    format!("{} {}", subtask_name.as_str(), task_args_parsed.as_str());
                let arg_matches_subtask = self
                    .clap_config
                    .to_owned()
                    .get_matches_from(complete_args_parsed.split(" "));
                self.update_variables_from_arg_matches(
                    &arg_matches_subtask
                        .subcommand_matches(&subtask_name)
                        .unwrap(),
                );
                self.get_command_string_parsed(subtask)
            }
            false => task.unparsed_commands.to_string(),
        };
        return self.replace_string_with_args(command_string);
    }

    fn replace_string_with_args(&self, string: String) -> String {
        let mut new_string = string;
        for (key, value) in self.variable_lookup.iter() {
            new_string = new_string.replace(&format!("${{{}}}", key), value);
        }
        new_string
    }
    pub fn setup_task_environment(&mut self, initial_arg_matches: ArgMatches) {
        // TODOL: make this a function
        let task_context_name = initial_arg_matches.get_one::<String>("context");
        // we can be confident in unwraps since we verify most values above on load
        let raw_args: Vec<_> = initial_arg_matches
            .get_many::<String>("task_info")
            .unwrap()
            .collect();

        let cli_inputs = self.clap_config.to_owned().get_matches_from(raw_args);
        // get matches found so far and parse into subcommand
        let (subcommand_name, clap_matched_args) = cli_inputs.subcommand().unwrap();
        let selected_task = self.get_config().to_owned();
        let selected_task = selected_task.get_task_by_name(subcommand_name).unwrap();
        let task_context = self.config.get_context(task_context_name);

        // TODO: make this a function
        // priority of variables being set
        // 1. cli input
        // 2. subtask input
        // 3. context
        // 4. defaults
        self.update_variables_from_task_stanza(selected_task.to_owned());
        if self.is_subtask(selected_task.to_owned()) {
            let (_, subtask) = &self.get_subtask(selected_task.to_owned());
            self.update_variables_from_task_stanza(subtask.to_owned());
        }

        // 2. context
        self.update_variables_from_context(task_context);

        // 1. cli input
        self.update_variables_from_arg_matches(
            &cli_inputs.subcommand_matches(&subcommand_name).unwrap(),
        );
        let parsed_command = self.get_command_string_parsed(selected_task.to_owned());
        println!("{}", parsed_command);
    }

}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::test_helpers::load_from_string;

    use super::TaskRunner;
    use clap::{value_parser, Arg, Command};

    #[test]
    fn test_update_variables_from_arg_matches() {
        let mut runner = TaskRunner::new(load_from_string());
        let args = Command::new("tester").arg(
            Arg::new("test")
                .required(true)
                .value_parser(value_parser!(String)),
        );
        let arg_matches = args.get_matches_from(vec!["tester", "test"]);
        runner.update_variables_from_arg_matches(&arg_matches);
        assert_eq!(runner.variable_lookup.get("test").unwrap(), "test");
    }

    #[test]
    fn test_update_variables_from_hashmap() {
        let mut runner = TaskRunner::new(load_from_string());
        let mut context = HashMap::new();
        context.insert("test".to_string(), "test".to_string());
        runner.update_variables_from_context(context);
        assert_eq!(runner.variable_lookup.get("test").unwrap(), "test");
    }
}
