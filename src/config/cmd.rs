use serde::Deserialize;

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

// cmd arg stanzas
#[derive(Deserialize, Clone)]
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
    pub fn get_name(&self) -> &str {
        return &self.name;
    }
    pub fn get_default(&self) -> Option<&str> {
        return self.default.as_deref();
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_cmd_arg() {
        let arg = super::CmdArg {
            name: "test".to_string(),
            default: Some("default".to_string()),
            arg_type: "string".to_string(),
        };
        let clap_arg = arg.get_clap_arg();
        assert_eq!(clap_arg.get_id(), "test");
        assert_eq!(clap_arg.is_required_set(), false);
        assert_eq!(clap_arg.get_default_values(), &["default"]);
    }
    #[test]
    fn test_display_arg_error() {
        let arg_error = super::ArgError {
            message: "test".to_string(),
        };
        assert_eq!(arg_error.to_string(), "Missing Value: test");
    }
}
