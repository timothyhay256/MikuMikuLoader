use serde::{Deserialize, Serialize};

use crate::scenario::ScenarioAdapter;

#[derive(Debug, Deserialize, Serialize)]
pub enum ModType {
    Story(ScenarioAdapter),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModData {
    pub mod_type: ModType,
}
