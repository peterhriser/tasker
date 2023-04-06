pub mod iters;
pub mod strings;
pub mod test_helpers;

use std::{
    io::{stdout, BufRead, BufReader, Write},
    process::{Command, Stdio},
};

use self::strings::split_exclude_quotes;

pub fn parse_task_name_from_string(parsed_command: &String) -> String {
    return split_exclude_quotes(parsed_command.to_string())[0].to_string();
}
pub fn parse_task_args_from_string(parsed_command: &String) -> Vec<String> {
    return split_exclude_quotes(parsed_command.to_string())[1..].to_vec();
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
mod unittest {
    use crate::utils::{split_exclude_quotes, strings::parse_command_from_string};

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
        let cmd =
            parse_command_from_string("echo \"beginning is here\" \"end is here\"".to_string());
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
