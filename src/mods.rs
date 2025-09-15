use std::{
    collections::HashMap,
    fs::{File, read_to_string},
    io::{Read, Write},
};

use anyhow::{Context, Result};
use log::{debug, error, warn};
use sekai_injector::{Config as SIConfig, InjectionMap};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{scenario::Scenario, utils::Config};

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

/// Walks through mod dir and updates injections-ab.toml with necessary paths
pub fn reload_injections(config: &Config) -> Result<()> {
    debug!("extracting sekai_injector config");
    let sekai_injector_conf: SIConfig =
        match File::open(config.advanced.sekai_injector_config_path.clone()) {
            Ok(mut file) => {
                let mut sekai_injector_conf_contents = String::new();
                file.read_to_string(&mut sekai_injector_conf_contents)
                    .expect("Config contains non UTF-8 characters.");

                toml::from_str(&sekai_injector_conf_contents).expect(
                    "The Sekai Injector config was not formatted properly and could not be read",
                )
            }
            Err(_) => {
                error!("No valid Sekai Injector config found, using default!");
                SIConfig::default()
            }
        };

    let resource_config_path = sekai_injector_conf
            .domains
            .into_iter()
            .find(|domain| domain.address == "assetbundle.sekai-en.com").expect("No config for assetbundle.sekai-en.com found in Sekai Injector config! Please fix or redownload the config.").resource_config;

    debug!("building injection map");
    let mut injection_map: InjectionMap = match File::open(&resource_config_path) {
        Ok(mut file) => {
            let mut injection_map_file_contents = String::new();
            file.read_to_string(&mut injection_map_file_contents).expect("The config file contains non UTF-8 characters, what in the world did you put in it??");
            toml::from_str(&injection_map_file_contents)
                .expect("The config file was not formatted properly and could not be read.")
        }
        Err(_) => {
            error!("No valid injection map found, using empty map!");
            InjectionMap { map: Vec::new() }
        }
    };

    debug!("walking through mods");
    for entry in WalkDir::new("mods") {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file()
                    && entry.path().extension().and_then(|e| e.to_str()) == Some("toml")
                {
                    debug!("trying {}", entry.path().display());
                    let entry_data = read_to_string(entry.path()).unwrap_or_else(|_| {
                            panic!(
                                "Could not read {}! Please try redownloading mods and fixing permissions.",
                                entry.path().display()
                            )
                        });

                    let mod_data: ModData = toml::from_str(&entry_data).unwrap_or_else(|_| {
                        panic!(
                            "{} is not formatted properly! Check if MikuMikuLoader is out of date.",
                            entry.path().display()
                        )
                    });

                    for injection in mod_data.injected_assets {
                        let new_injection = (injection.0.clone(), injection.1, true);

                        if injection_map
                            .map
                            .iter()
                            .any(|existing_injection| existing_injection.0 == injection.0)
                        {
                            warn!("Existing injection for {} exists, skipping it", injection.0)
                        } else if !injection_map.map.contains(&new_injection) {
                            injection_map.map.push(new_injection);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Couldn't open an mod: {e}")
            }
        }
    }

    debug!("Saving injection map");
    let injection_map_toml = toml::to_string_pretty(&injection_map)
        .context("Error converting injection_map into toml")?;

    let mut resource_config_file = File::create(&resource_config_path)
        .context(format!("Error opening {resource_config_path}"))?;
    resource_config_file
        .write_all(injection_map_toml.as_bytes())
        .context(format!("Error writing to {resource_config_path}"))?;
    resource_config_file.flush()?;

    Ok(())
}
