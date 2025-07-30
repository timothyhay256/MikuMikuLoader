use std::ffi::CString;

use pyo3::prelude::*;
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
