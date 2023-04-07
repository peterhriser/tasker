use std::{
    io::{stdout, BufRead, BufReader, Write},
    process::{Command, Stdio},
};

pub struct TaskRunner {
    commands: Vec<String>,
}

impl TaskRunner {
    pub fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }
    pub fn call_command(mut command: Command) -> Result<(), Box<dyn std::error::Error>> {
        let cmd_stdout = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
            .stdout
            .unwrap();

        let reader = BufReader::new(cmd_stdout);

        BufRead::lines(reader).for_each(|line| {
            Write::flush(&mut stdout()).unwrap();
            println!("> {}", line.unwrap())
        });

        Ok(())
    }
    pub fn parse_command_from_string(command: String) -> Command {
        let mut parts = command.split_whitespace();
        let command_name = parts.next().expect("no command specified");
        let args = parts;

        let mut cmd = Command::new(command_name);
        cmd.args(args);
        cmd
    }
    fn parse_strings_into_commands(&self) -> Vec<Command> {
        let commands = self.commands.clone();
        return commands
            .into_iter()
            .map(|cmd| Self::parse_command_from_string(cmd))
            .collect();
    }
    pub fn execute_tasks(&self) {
        let commands = self.parse_strings_into_commands();
        for cmd in commands {
            Self::call_command(cmd).expect("Error Executing Command");
        }
    }
    pub fn print_commands(&self) {
        for i in 0..self.commands.len() {
            println!("{:?}: {}", i, self.commands[i]);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::runners::runner::TaskRunner;

    #[test]
    fn test_parse_command_from_string() {
        let cmd = TaskRunner::parse_command_from_string(
            "echo \"beginning is here\" \"end is here\"".to_string(),
        );
        assert_eq!(cmd.get_program(), "echo");
        let args = cmd.get_args();
        let mut arg_list = vec![];
        for arg in args {
            arg_list.push(arg.to_string_lossy().to_string());
        }
        assert_eq!(
            arg_list,
            vec!["\"beginning", "is", "here\"", "\"end", "is", "here\""]
        );
    }
}
