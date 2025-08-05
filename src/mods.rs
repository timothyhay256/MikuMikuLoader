use std::ffi::CString;

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{scenario::ScenarioAdapter, utils::Config};

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
    /// Name of the mod
    pub mod_name: String,
    /// Is the mod active? Set by user
    pub enabled: bool,
    /// Enum containing all possible types of mods
    pub mod_type: ModType,
    /// Set game resource paths you want to force to be redownloaded here. By default, all injected resources are invalidated initially.
    pub invalidated_assets: Vec<InvalidateCacheEntry>,
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

pub fn create_assetbundle(modpack: ModData) -> PyResult<()> {
    match modpack.mod_type {
        ModType::Story(scenario_adapter) => Python::with_gil(|py| {
            const PY_CODE: &str = include_str!("../python/story_to_assetbundle.py");

            Python::with_gil(|py| {
                let filename = CString::new("story_to_assetbundle.py").unwrap();
                let modname = CString::new("story_to_assetbundle").unwrap();

                let module =
                    PyModule::from_code(py, &CString::new(PY_CODE).unwrap(), &filename, &modname)?;

                let func = module.getattr("list_assets")?;
                func.call1(("assets/template",))?;

                Ok(())
            })
        }),
    }
}

pub fn reload_mods(config: Config) {
    todo!()
}
