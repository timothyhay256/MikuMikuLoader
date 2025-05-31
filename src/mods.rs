use serde::Deserialize;

use crate::scenario::ScenarioAdapter;

#[derive(Debug, Deserialize)]
pub enum ModType {
    Story(ScenarioAdapter),
}

#[derive(Debug, Deserialize)]
pub struct ModData {
    pub mod_type: ModType,
}
