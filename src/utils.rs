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
    pub live2d_asset_url: String,
    pub needed_asset_files: Vec<String>,
    pub needed_template_files: Vec<String>,
    pub needed_live2d_files: Vec<String>,
}

impl Default for AssetConfig {
    fn default() -> Self {
        AssetConfig {
            common_asset_url:
                "raw.githubusercontent.com/Sekai-World/sekai-master-db-diff/refs/heads/main"
                    .to_string(),
            template_asset_url:
                "raw.githubusercontent.com/timothyhay256/MML-templates/refs/heads/main".to_string(),
            live2d_asset_url: "sekai-stories.pages.dev".to_string(),
            needed_asset_files: vec!["/character2ds.json".to_string()],
            needed_template_files: vec!["/story/scenario/template.yaml".to_string()],
            needed_live2d_files: vec![
                String::from("01_ichika/01ichika_motion_base/BuildMotionData.json"),
                String::from("02_saki/02saki_motion_base/BuildMotionData.json"),
                String::from("03_honami/03honami_motion_base/BuildMotionData.json"),
                String::from("04_shiho/04shiho_motion_base/BuildMotionData.json"),
                String::from("05_minori/05minori_motion_base/BuildMotionData.json"),
                String::from("06_haruka/06haruka_motion_base/BuildMotionData.json"),
                String::from("07_airi/07airi_motion_base/BuildMotionData.json"),
                String::from("08_shizuku/08shizuku_motion_base/BuildMotionData.json"),
                String::from("09_kohane/09kohane_motion_base/BuildMotionData.json"),
                String::from("10_an/10an_motion_base/BuildMotionData.json"),
                String::from("11_akito/11akito_motion_base/BuildMotionData.json"),
                String::from("12_touya/12touya_motion_base/BuildMotionData.json"),
                String::from("13_tsukasa/13tsukasa_motion_base/BuildMotionData.json"),
                String::from("14_emu/14emu_motion_base/BuildMotionData.json"),
                String::from("15_nene/15nene_motion_base/BuildMotionData.json"),
                String::from("16_rui/16rui_motion_base/BuildMotionData.json"),
                String::from("17_kanade/17kanade_motion_base/BuildMotionData.json"),
                String::from("18_mafuyu/18mafuyu_motion_base/BuildMotionData.json"),
                String::from("19_ena/19ena_motion_base/BuildMotionData.json"),
                String::from("20_mizuki/20mizuki_motion_base/BuildMotionData.json"),
                String::from("21_miku/21miku_motion_base/BuildMotionData.json"),
                String::from("22_rin/22rin_motion_base/BuildMotionData.json"),
                String::from("23_len/23len_motion_base/BuildMotionData.json"),
                String::from("24_luka/24luka_motion_base/BuildMotionData.json"),
                String::from("25_meiko/25meiko_motion_base/BuildMotionData.json"),
                String::from("26_kaito/26kaito_motion_base/BuildMotionData.json"),
            ],
        }
    }
}
