#[cfg(test)]
pub mod test_helpers {
    use crate::taskfile::Taskfile;
    pub fn load_from_string() -> Taskfile {
        let example_file = r#"project: "Example"
version: "1.0"
author: "Peter"
contexts:
  test:
    test_key: test_value
tasks:
  - name: test-cmd
    commands:
    - shell: echo Hello ${required_arg} ${default_arg}
    description: "greets a user by name"
    args:
      - name: required_arg
        type: string
      - name: default_arg
        type: string
        default: default
  - name: test-task
    commands:
    - task: test-cmd Foo Bar
    - task: test-cmd Bar Foo
    description: "greets a user by name"
    args:
"#;
        return serde_yaml::from_str(example_file).unwrap();
    }
}
