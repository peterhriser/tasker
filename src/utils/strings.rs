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
