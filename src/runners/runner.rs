use std::collections::HashMap;

use clap::ArgMatches;

use crate::{
    config::{taskfile::Taskfile, taskstanza::TaskStanza},
    utils::{call_command, parse_command_from_string, split_exclude_quotes},
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
    fn get_subtask(&self, task: &TaskStanza) -> Option<(String, TaskStanza)> {
        let subtask = self.config.get_subtask(task);
        match subtask {
            Ok((name, task)) => Some((name, task)),
            Err(_) => None,
        }
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

    fn get_command_string_parsed(&mut self, task: TaskStanza) -> String {
        let command_string = match task.is_subtask() {
            true => {
                let raw_cmd = task.unparsed_commands.to_string();
                let (subtask_name, subtask) = self.get_subtask(&task).unwrap();
                let task_args = raw_cmd
                    .split(" ")
                    .collect::<Vec<&str>>()
                    .split_at(1)
                    .1
                    .join(" ");
                let task_args_parsed = self.replace_string_with_args(task_args);
                let complete_args_parsed =
                    format!("{} {}", subtask_name.as_str(), task_args_parsed.as_str());
                println!("before_get_matches -> {}", complete_args_parsed.as_str());
                let arg_matches_subtask = self
                    .clap_config
                    .to_owned()
                    .get_matches_from(split_exclude_quotes(complete_args_parsed));
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
            if value.contains(" ") {
                new_string =
                    new_string.replace(&format!("${{{}}}", key), &format!("\"{}\"", value));
                continue;
            } else {
                new_string = new_string.replace(&format!("${{{}}}", key), value);
            }
        }
        new_string
    }
    fn load_variables(
        &mut self,
        selected_task: &TaskStanza,
        task_name: String,
        selected_context: HashMap<String, String>,
        cli_inputs: ArgMatches,
    ) {
        // 3. defaults
        self.update_variables_from_task_stanza(selected_task.to_owned());
        if selected_task.is_subtask() {
            let (_, subtask) = self.get_subtask(selected_task).unwrap();
            self.update_variables_from_task_stanza(subtask.to_owned());
        }
        // 2. context
        self.update_variables_from_context(selected_context);

        // 1. cli input
        self.update_variables_from_arg_matches(&cli_inputs.subcommand_matches(&task_name).unwrap());
    }
    fn parse_cli_inputs(
        &mut self,
        initial_arg_matches: ArgMatches,
    ) -> (String, Option<String>, ArgMatches) {
        // TODO: make this a function
        let context_name = initial_arg_matches.get_one::<String>("context");
        let context_name = match context_name {
            Some(name) => Some(name.to_owned()),
            None => None,
        };
        // we can be confident in unwraps since we verify most values above on load
        let raw_args: Vec<_> = initial_arg_matches
            .get_many::<String>("task_info")
            .unwrap()
            .collect();

        let cli_inputs = self.clap_config.to_owned().get_matches_from(raw_args);
        // get matches found so far and parse into subcommand
        let (task_name, _) = cli_inputs.subcommand().unwrap();
        let task_name = task_name.to_string().to_owned();
        return (task_name, context_name, cli_inputs);
    }
    pub fn gather_task_info_from_cli(
        &mut self,
        task_name: &String,
        context_name: Option<String>,
    ) -> (TaskStanza, HashMap<String, String>) {
        // TODO: make this a function
        let cfg = self.get_config().to_owned();
        let selected_task = cfg.get_task_by_name(task_name).unwrap();
        let task_context = self.config.get_context(context_name);
        return (selected_task.to_owned(), task_context.to_owned());
    }

    pub fn execute_task(&mut self, initial_arg_matches: ArgMatches) {
        let (task_name, context_name, cli_inputs) = self.parse_cli_inputs(initial_arg_matches);
        let (selected_task, selected_context) =
            self.gather_task_info_from_cli(&task_name, context_name);

        self.load_variables(&selected_task, task_name, selected_context, cli_inputs);
        let parsed_command = self.get_command_string_parsed(selected_task);
        let command = parse_command_from_string(parsed_command);
        call_command(command).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::TaskRunner;
    use crate::test_helpers::load_from_string;
    use clap::{value_parser, Arg, Command};
    use std::collections::HashMap;

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
    #[test]
    fn test_update_variables_from_task_stanza() {
        let mut runner = TaskRunner::new(load_from_string());
        let task = runner.get_config().get_task_by_name("test-cmd").unwrap();
        runner.update_variables_from_task_stanza(task.to_owned());
        assert_eq!(runner.variable_lookup.get("last").unwrap(), "default");
    }
    #[test]
    fn test_replace_string_with_args() {
        let mut runner = TaskRunner::new(load_from_string());
        let map: HashMap<String, String> =
            HashMap::from([("test".to_string(), "test".to_string())]);
        runner.update_variables_from_context(map);
        let new_string = runner.replace_string_with_args("test ${test}".to_string());
        assert_eq!(new_string, "test test");
    }
    #[test]
    fn test_get_command_string_parsed() {
        let mut runner = TaskRunner::new(load_from_string());
        let task = runner
            .get_config()
            .get_task_by_name("test-cmd")
            .unwrap()
            .to_owned();

        let mut context = HashMap::new();
        context.insert("first".to_string(), "first".to_string());
        runner.update_variables_from_context(context);
        runner.update_variables_from_task_stanza(task.to_owned());

        let new_string = runner.get_command_string_parsed(task.to_owned());
        assert_eq!(new_string, "echo first default");
    }
    #[test]
    fn test_get_command_string_parsed_subtask() {
        let mut runner = TaskRunner::new(load_from_string());
        let task = runner
            .get_config()
            .get_task_by_name("test-task")
            .unwrap()
            .to_owned();

        runner.update_variables_from_task_stanza(task.to_owned());

        let new_string = runner.get_command_string_parsed(task.to_owned());
        assert_eq!(new_string, "echo beginning end");
    }
}
