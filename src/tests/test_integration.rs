#[cfg(test)]
mod integration_tests {
    use clap::CommandFactory;

    use crate::{run_from_matches, CliArgs};

    #[test]
    fn test_entry_point() {
        let initial_arg_matches = CliArgs::command().get_matches_from(vec![
            "tasker",
            "-c",
            "src/tests/Taskfile",
            "greet",
            "Peter",
        ]);
        let result = run_from_matches(initial_arg_matches);
        assert!(result.is_ok())
    }
    #[test]
    fn test_missing_file() {
        let initial_arg_matches =
            CliArgs::command().get_matches_from(vec!["tasker", "-c", "fakefile", "hello", "Peter"]);
        let result = run_from_matches(initial_arg_matches);
        assert!(result.is_err())
    }
    #[test]
    fn test_print_error() {
        let initial_arg_matches = CliArgs::command().get_matches_from(vec!["tasker"]);
        let result = run_from_matches(initial_arg_matches);
        assert!(result.is_err())
    }
}
