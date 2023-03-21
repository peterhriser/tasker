use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct TaskContext {
    pub vars: HashMap<String, String>,
}