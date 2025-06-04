use serde::{Deserialize, Serialize};

use crate::scenario::ScenarioAdapter;

#[derive(Debug, Deserialize, Serialize)]
pub enum ModType {
    Story(ScenarioAdapter),
}

impl ModType {
    pub fn variant_name(&self) -> &'static str {
        match self {
            ModType::Story(_) => "Story",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModData {
    pub mod_name: String,
    pub enabled: bool,
    pub mod_type: ModType,
}
