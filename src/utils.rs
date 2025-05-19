use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub advanced: AdvancedConfig,
}

#[derive(Deserialize)]
pub struct AdvancedConfig {
    pub sekai_injector_config_path: String,
}
