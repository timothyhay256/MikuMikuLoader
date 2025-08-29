use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::scenario::Scenario;

#[derive(Debug, Deserialize, Serialize)]
pub enum ModType {
    Story(Scenario),
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
    /// Name of the mod
    pub mod_name: String,
    /// Is the mod active? Set by user
    pub enabled: bool,
    /// Enum containing all possible types of mods
    pub mod_type: ModType,
    /// Set game resource paths you want to force to be redownloaded here. By default, all injected resources are invalidated initially.
    pub invalidated_assets: Vec<InvalidateCacheEntry>,
    /// HashMap containing all assets to be injected. Key is resource path to override, value is path to local AssetBundle file.
    pub injected_assets: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CacheInvalidDuration {
    /// Invalidated on the first mod reload only
    InitiallyInvalid,
    /// Invalidated on every mod reload
    PermanentlyInvalid,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InvalidateCacheEntry {
    pub resource_path: String,
    pub duration: CacheInvalidDuration,
}
