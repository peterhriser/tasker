use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug)]
pub(super) struct ArgError {
    pub(crate) message: String,
}

impl std::error::Error for ArgError {}

impl std::fmt::Display for ArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing Value: {}", self.message)
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AllowedVarTypes {
    U(u64),
    S(String),
    V(Vec<AllowedVarTypes>),
}

impl fmt::Display for AllowedVarTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AllowedVarTypes::U(u) => write!(f, "{}", u),
            AllowedVarTypes::S(s) => write!(f, "{}", s),
            AllowedVarTypes::V(v) => {
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
        }
    }
}

// cmd arg stanzas
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CmdArg {
    pub name: String,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(rename = "type")]
    pub arg_type: String,
}
impl CmdArg {
    fn is_required(&self) -> bool {
        if self.default.is_none() {
            return true;
        }
        return false;
    }
    pub(super) fn get_clap_arg(&self) -> clap::Arg {
        let name_owned = self.name.to_owned();
        if !self.is_required() {
            let default = self.default.to_owned().unwrap();
            return clap::Arg::new(name_owned).default_value(default);
        } else {
            return clap::Arg::new(name_owned).required(true);
        }
    }
    pub(super) fn set_default_from_option(&mut self, new_default: Option<String>) {
        match new_default {
            Some(item) => {
                let copied_val = item.to_string();
                self.default = Some(copied_val);
            }
            None => {}
        };
    }
}

#[cfg(test)]
pub mod cmd_test_helpers {
    use super::CmdArg;

    pub fn create_cmd_arg_for_test(required: bool) -> CmdArg {
        if required {
            return CmdArg {
                name: "required_arg".to_string(),
                default: None,
                arg_type: "string".to_string(),
            };
        } else {
            return CmdArg {
                name: "optional_arg".to_string(),
                default: Some("DefaultValue".to_string()),
                arg_type: "string".to_string(),
            };
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::config::cmd::cmd_test_helpers::create_cmd_arg_for_test;

    #[test]
    fn test_is_required() {
        let required_arg = create_cmd_arg_for_test(true);
        assert!(required_arg.is_required());

        let optional_arg = create_cmd_arg_for_test(false);
        assert!(!optional_arg.is_required());
    }
    #[test]
    fn test_set_new_default() {
        let mut required_arg = create_cmd_arg_for_test(true);
        assert!(required_arg.is_required());

        // setting a default will make the arg no longer required
        required_arg.set_default_from_option(Some("new_default".to_string()));
        assert!(!required_arg.is_required());
    }
    #[test]
    fn test_get_clap_arg_no_default() {
        let required_arg = create_cmd_arg_for_test(true);
        let clap_arg = required_arg.get_clap_arg();
        assert!(clap_arg.is_required_set());
    }
    #[test]
    fn test_get_clap_arg_with_default() {
        let optional_arg = create_cmd_arg_for_test(false);
        let clap_arg = optional_arg.get_clap_arg();
        assert!(!clap_arg.is_required_set());
    }
}