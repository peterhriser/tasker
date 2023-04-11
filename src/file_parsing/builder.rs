use std::collections::HashMap;

use clap::ArgMatches;

use crate::{
    file_parsing::{taskfile::Taskfile, taskstanza::TaskStanza},
    utils::{iters::upsert_into_hash_map, strings::split_exclude_quotes},
};

use super::runner::TaskRunner;
pub struct TaskBuilder {
    config: Taskfile,
    variable_lookup: HashMap<String, String>,
    clap_config: clap::Command,
}

impl TaskBuilder {
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

    // used for getting defaults and subtask values
    fn update_variables_from_task_stanza(&mut self, task: TaskStanza) {
        let mut local_variable_lookup = self.variable_lookup.clone();
        for cmd in task.get_command_args() {
            let value = match cmd.get_default() {
                Some(value) => value,
                None => continue,
            };
            let key = cmd.get_name();
            upsert_into_hash_map(key.to_string(), value.to_string(), &mut local_variable_lookup);
        }
        self.variable_lookup = local_variable_lookup;
    }
    fn update_variables_from_arg_matches(&mut self, args: &ArgMatches) {
        let mut local_variable_lookup = self.variable_lookup.clone();
        for id in args.ids() {
            let key = id.to_string();
            let value = args.get_one::<String>(id.as_str()).unwrap().to_string();
            upsert_into_hash_map(key, value, &mut local_variable_lookup);
        }
        self.variable_lookup = local_variable_lookup;
    }
    fn update_variables_from_context(&mut self, context: HashMap<String, String>) {
        let mut local_variable_lookup = self.variable_lookup.clone();
        for (key, value) in context.iter() {
            upsert_into_hash_map(
                key.to_string(),
                value.to_string(),
                &mut local_variable_lookup,
            );
        }
        self.variable_lookup = local_variable_lookup;
    }

    fn replace_string_with_args(string: String, local_vars: &HashMap<String, String>) -> String {
        let mut new_string = string;
        for (key, value) in local_vars.iter() {
            if value.contains(" ") {
                new_string = new_string.replace(&format!("${{{}}}", key), &format!("{}", value));
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

    pub fn create_command_strings(&mut self, initial_arg_matches: ArgMatches) -> Vec<String> {
        let (task_name, context_name, cli_inputs) = self.parse_cli_inputs(initial_arg_matches);
        let (selected_task, selected_context) =
            self.gather_task_info_from_cli(&task_name, context_name);

        self.load_variables(&selected_task, task_name, selected_context, cli_inputs);
        let cloned_vars = self.variable_lookup.clone();
        return self.get_all_commands_parsed(selected_task, cloned_vars);
    }

    fn get_all_commands_parsed(
        &self,
        task: TaskStanza,
        current_variables: HashMap<String, String>,
    ) -> Vec<String> {
        // return a list of filled in commands for a given task

        let mut commands: Vec<String> = Vec::new();
        let mut local_vars = current_variables.to_owned();
        for cmd in task.commands {
            let command_type = cmd.key.to_owned();
            let raw_command = cmd.value.to_owned();
            match command_type {
                // base case
                crate::file_parsing::cmd::CommandTypes::Shell(_) => {
                    let parsed_command = Self::replace_string_with_args(raw_command, &local_vars);
                    commands.push(parsed_command)
                }
                crate::file_parsing::cmd::CommandTypes::Task(_) => {
                    // fill in variables, then recurse through the subtask
                    let parsed_command = Self::replace_string_with_args(raw_command, &local_vars);
                    let sub_task_name: String = Self::parse_task_name_from_string(&parsed_command);
                    let sub_task_supplied_args: Vec<String> =
                        Self::parse_task_args_from_string(&parsed_command);
                    let sub_task = self.config.get_task_by_name(&sub_task_name).unwrap();
                    let sub_task_expected_args = sub_task.get_command_args();
                    for i in 0..sub_task_expected_args.len() {
                        let arg = &sub_task_expected_args[i];
                        let key = arg.get_name();
                        let value = match sub_task_supplied_args.get(i) {
                            Some(_) => sub_task_supplied_args[i].to_string(),
                            None => {
                                // TODO: handle error here for missing argument
                                sub_task_expected_args[i].get_default().unwrap().to_string()
                            }
                        };
                        upsert_into_hash_map(
                            key.to_string().to_owned(),
                            value.to_owned(),
                            &mut local_vars,
                        );
                    }
                    commands.extend(
                        self.get_all_commands_parsed(sub_task.to_owned(), local_vars.to_owned()),
                    );
                }
                crate::file_parsing::cmd::CommandTypes::Script(_) => unimplemented!(),
            }
        }
        return commands;
    }
    pub fn create_task_runner(&mut self, initial_arg_matches: ArgMatches) -> TaskRunner {
        let commands = self.create_command_strings(initial_arg_matches);
        let task_runner = TaskRunner::new(commands);
        return task_runner;
    }
    pub fn parse_task_name_from_string(parsed_command: &String) -> String {
        return split_exclude_quotes(parsed_command.to_string())[0].to_string();
    }
    pub fn parse_task_args_from_string(parsed_command: &String) -> Vec<String> {
        return split_exclude_quotes(parsed_command.to_string())[1..].to_vec();
    }
}
#[cfg(test)]
mod tests {
    use super::TaskBuilder;
    use crate::utils::test_helpers::test_helpers::load_from_string;
    use clap::{value_parser, Arg, Command};
    use std::collections::HashMap;

    #[test]
    fn test_update_variables_from_arg_matches() {
        let mut runner = TaskBuilder::new(load_from_string());
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
        let mut runner = TaskBuilder::new(load_from_string());
        let mut context = HashMap::new();
        context.insert("test".to_string(), "test".to_string());
        runner.update_variables_from_context(context);
        assert_eq!(runner.variable_lookup.get("test").unwrap(), "test");
    }
    #[test]
    fn test_update_variables_from_task_stanza() {
        let mut runner = TaskBuilder::new(load_from_string());
        let task = runner.get_config().get_task_by_name("test-cmd").unwrap();
        runner.update_variables_from_task_stanza(task.to_owned());
        assert_eq!(
            runner.variable_lookup.get("default_arg").unwrap(),
            "default"
        );
    }
    #[test]
    fn test_replace_string_with_args() {
        let mut runner = TaskBuilder::new(load_from_string());
        let map: HashMap<String, String> =
            HashMap::from([("test".to_string(), "test".to_string())]);
        runner.update_variables_from_context(map);
        let new_string = TaskBuilder::replace_string_with_args(
            "test ${test}".to_string(),
            &runner.variable_lookup,
        );
        assert_eq!(new_string, "test test");
    }
    #[test]
    fn test_parse_task_name_from_string() {
        let task_name = TaskBuilder::parse_task_name_from_string(&"test".to_string());
        assert_eq!(task_name, "test");
    }
    #[test]
    fn test_get_all_commands_parsed_with_task() {
        let runner = TaskBuilder::new(load_from_string());
        let task = runner.get_config().get_task_by_name("test-task").unwrap();
        let commands = runner.get_all_commands_parsed(task.to_owned(), HashMap::new());
        assert_eq!(commands[0], "echo Hello Foo Bar");
        assert_eq!(commands[1], "echo Hello Bar Foo");
    }
}
