// TODO make this a crate
pub mod errors;
pub mod iters;
pub mod strings;
pub mod test_helpers;

#[cfg(test)]
mod unittest {
    use crate::utils::strings::split_exclude_quotes;

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
}
