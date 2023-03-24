use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct TaskContext {
    pub vars: HashMap<String, String>,
}
