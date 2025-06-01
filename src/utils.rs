use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config {
    pub advanced: AdvancedConfig,
}

#[derive(Deserialize)]
pub struct AdvancedConfig {
    pub sekai_injector_config_path: String,
    pub assets: AssetConfig,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        AdvancedConfig {
            sekai_injector_config_path: "sekai-injector.toml".to_string(),
            assets: AssetConfig::default(),
        }
    }
}

#[derive(Deserialize)]
pub struct AssetConfig {
    pub common_asset_url: String,
    pub template_asset_url: String,
    pub needed_asset_files: Vec<String>,
    pub needed_template_files: Vec<String>,
}

impl Default for AssetConfig {
    fn default() -> Self {
        AssetConfig {
            common_asset_url:
                "raw.githubusercontent.com/Sekai-World/sekai-master-db-diff/refs/heads/main"
                    .to_string(),
            template_asset_url:
                "raw.githubusercontent.com/timothyhay256/MML-templates/refs/heads/main".to_string(),
            needed_asset_files: vec!["/character2ds.json".to_string()],
            needed_template_files: vec!["/story/scenario/template.yaml".to_string()],
        }
    }
}
