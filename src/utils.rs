use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config {
    pub advanced: AdvancedConfig,
}

#[derive(Deserialize)]
pub struct AdvancedConfig {
    pub sekai_injector_config_path: String,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        AdvancedConfig {
            sekai_injector_config_path: "sekai-injector.toml".to_string(),
        }
    }
}
