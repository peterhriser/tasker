use std::{
    io::{stdout, BufRead, BufReader, Write},
    process::{Command, Stdio},
};

pub fn split_exclude_quotes(s: String) -> Vec<String> {
    let mut split = vec![];
    let mut current = String::new();
    let mut in_quotes = false;
    for c in s.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c == ' ' && !in_quotes {
            split.push(current.clone().to_string());
            current = String::new();
        } else {
            current.push(c);
        }
    }
    split.push(current.clone().to_string());
    split
}
pub fn parse_command_from_string(command: String) -> Command {
    // todo: move to util module
    let mut parts = command.split_whitespace();
    let command_name = parts.next().expect("no command specified");
    let args = parts;

    let mut cmd = Command::new(command_name);
    cmd.args(args);
    cmd
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
#[cfg(test)]
mod test {
    use crate::utils::split_exclude_quotes;

    #[test]
    fn test_split_exclude_quotes() {
        let spl = split_exclude_quotes("echo \"beginning is here\" end".to_string());
        assert_eq!(vec!["echo", "beginning is here", "end"], spl);
    }
    #[test]
    fn test_split_exclude_quotes_2() {
        let spl = split_exclude_quotes("echo \"beginning is here\" \"end is here\"".to_string());
        assert_eq!(vec!["echo", "beginning is here", "end is here"], spl);
    }
    #[test]
    fn test_parse_command_from_string() {
        let cmd = super::parse_command_from_string(
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
