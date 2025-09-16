use std::collections::HashMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct Config {
    pub advanced: AdvancedConfig,
    pub platform: String,
    pub region: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            advanced: AdvancedConfig::default(),
            platform: "android".to_string(),
            region: "en".to_string(),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct AdvancedConfig {
    pub sekai_injector_config_path: String,
    pub assetbundle_url: String,
    pub assetbundle_info_url: String,
    pub assets: AssetConfig,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        AdvancedConfig {
            sekai_injector_config_path: "sekai-injector.toml".to_string(),
            assetbundle_url: "assetbundle.sekai-en.com".to_string(),
            assetbundle_info_url: "assetbundle-info.sekai-en.com".to_string(),
            assets: AssetConfig::default(),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct AssetConfig {
    pub asset_path: String,
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
            asset_path: "assets".to_string(),
            common_asset_url:
                "raw.githubusercontent.com/Sekai-World/sekai-master-db-en-diff/refs/heads/main"
                    .to_string(),
            template_asset_url:
                "raw.githubusercontent.com/timothyhay256/MML-templates/refs/heads/main".to_string(),
            live2d_asset_url:
                "raw.githubusercontent.com/lezzthanthree/SEKAI-Stories/refs/heads/master"
                    .to_string(),
            needed_asset_files: vec![
                "/character2ds.json".to_string(),
                "/versions.json".to_string(),
            ],
            needed_template_files: vec![
                "/story/scenario/scenario".to_string(),
                "/story/screen_image/screen_image".to_string(),
                "/event/logo/logo".to_string(),
            ],
            needed_live2d_files: vec![
                String::from(
                    "/public/live2d/model/01ichika/01ichika_cloth001/01ichika_cloth001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/01ichika/01ichika_culture/01ichika_culture.model3.json",
                ),
                String::from("/public/live2d/model/01ichika/01ichika_jc/01ichika_jc.model3.json"),
                String::from(
                    "/public/live2d/model/01ichika/01ichika_normal/01ichika_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/01ichika/01ichika_sports/01ichika_sports.model3.json",
                ),
                String::from(
                    "/public/live2d/model/01ichika/01ichika_sports02/01ichika_sports02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/01ichika/01ichika_swim/01ichika_swim.model3.json",
                ),
                String::from(
                    "/public/live2d/model/01ichika/01ichika_unit/01ichika_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/01ichika/v2_01ichika_casual/v2_01ichika_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/01ichika/v2_01ichika_school01/v2_01ichika_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/01ichika/v2_01ichika_unit/v2_01ichika_unit.model3.json",
                ),
                String::from("/public/live2d/model/02saki/02saki_black/02saki_black.model3.json"),
                String::from(
                    "/public/live2d/model/02saki/02saki_cloth001/02saki_cloth001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/02saki/02saki_culture/02saki_culture.model3.json",
                ),
                String::from("/public/live2d/model/02saki/02saki_jc/02saki_jc.model3.json"),
                String::from("/public/live2d/model/02saki/02saki_normal/02saki_normal.model3.json"),
                String::from(
                    "/public/live2d/model/02saki/02saki_pajamas/02saki_pajamas.model3.json",
                ),
                String::from(
                    "/public/live2d/model/02saki/02saki_parttime/02saki_parttime.model3.json",
                ),
                String::from("/public/live2d/model/02saki/02saki_sports/02saki_sports.model3.json"),
                String::from(
                    "/public/live2d/model/02saki/02saki_sports02/02saki_sports02.model3.json",
                ),
                String::from("/public/live2d/model/02saki/02saki_swim/02saki_swim.model3.json"),
                String::from("/public/live2d/model/02saki/02saki_unit/02saki_unit.model3.json"),
                String::from(
                    "/public/live2d/model/02saki/v2_02saki_casual/v2_02saki_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/02saki/v2_02saki_parttime/v2_02saki_parttime.model3.json",
                ),
                String::from(
                    "/public/live2d/model/02saki/v2_02saki_school01/v2_02saki_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/02saki/v2_02saki_unit/v2_02saki_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/03honami/03honami_black/03honami_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/03honami/03honami_cloth001/03honami_cloth001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/03honami/03honami_culture/03honami_culture.model3.json",
                ),
                String::from("/public/live2d/model/03honami/03honami_jc/03honami_jc.model3.json"),
                String::from(
                    "/public/live2d/model/03honami/03honami_normal/03honami_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/03honami/03honami_sports/03honami_sports.model3.json",
                ),
                String::from(
                    "/public/live2d/model/03honami/03honami_sports02/03honami_sports02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/03honami/03honami_swim/03honami_swim.model3.json",
                ),
                String::from(
                    "/public/live2d/model/03honami/03honami_unit/03honami_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/03honami/03honami_wedding/03honami_wedding.model3.json",
                ),
                String::from(
                    "/public/live2d/model/03honami/v2_03honami_casual/v2_03honami_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/03honami/v2_03honami_school01/v2_03honami_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/03honami/v2_03honami_unit/v2_03honami_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/04shiho/04shiho_black/04shiho_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/04shiho/04shiho_cloth001/04shiho_cloth001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/04shiho/04shiho_culture/04shiho_culture.model3.json",
                ),
                String::from("/public/live2d/model/04shiho/04shiho_jc/04shiho_jc.model3.json"),
                String::from(
                    "/public/live2d/model/04shiho/04shiho_normal/04shiho_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/04shiho/04shiho_sports/04shiho_sports.model3.json",
                ),
                String::from(
                    "/public/live2d/model/04shiho/04shiho_sports02/04shiho_sports02.model3.json",
                ),
                String::from("/public/live2d/model/04shiho/04shiho_swim/04shiho_swim.model3.json"),
                String::from("/public/live2d/model/04shiho/04shiho_unit/04shiho_unit.model3.json"),
                String::from(
                    "/public/live2d/model/04shiho/04shiho_yumeyume/04shiho_yumeyume.model3.json",
                ),
                String::from(
                    "/public/live2d/model/04shiho/v2_04shiho_casual/v2_04shiho_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/04shiho/v2_04shiho_school01/v2_04shiho_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/04shiho/v2_04shiho_sports02/v2_04shiho_sports02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/04shiho/v2_04shiho_unit/v2_04shiho_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/05minori_cloth001/05minori_cloth001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/05minori_cloth002/05minori_cloth002.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/05minori_culture/05minori_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/05minori_fancostume/05minori_fancostume.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/05minori_normal/05minori_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/05minori_parttime/05minori_parttime.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/05minori_priestess/05minori_priestess.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/05minori_sports/05minori_sports.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/05minori_sports02/05minori_sports02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/05minori_swim/05minori_swim.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/05minori_unit/05minori_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/v2_05minori_casual/v2_05minori_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/v2_05minori_fancostume/v2_05minori_fancostume.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/v2_05minori_lesson/v2_05minori_lesson.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/v2_05minori_school01/v2_05minori_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/v2_05minori_sports02/v2_05minori_sports02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/v2_05minori_unit/v2_05minori_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/05minori/v2_05minori_wedding/v2_05minori_wedding.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/06haruka_asuran/06haruka_asuran.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/06haruka_black/06haruka_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/06haruka_cloth001/06haruka_cloth001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/06haruka_cloth002/06haruka_cloth002.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/06haruka_culture/06haruka_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/06haruka_fancostume/06haruka_fancostume.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/06haruka_normal/06haruka_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/06haruka_priestess/06haruka_priestess.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/06haruka_sanisani/06haruka_sanisani.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/06haruka_sports/06haruka_sports.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/06haruka_sports02/06haruka_sports02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/06haruka_swim/06haruka_swim.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/06haruka_unit/06haruka_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/v2_06haruka_casual/v2_06haruka_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/v2_06haruka_casual_black/v2_06haruka_casual_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/v2_06haruka_fancostume/v2_06haruka_fancostume.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/v2_06haruka_lesson/v2_06haruka_lesson.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/v2_06haruka_school01/v2_06haruka_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/06haruka/v2_06haruka_unit/v2_06haruka_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/07airi/07airi_cloth001/07airi_cloth001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/07airi/07airi_cloth002/07airi_cloth002.model3.json",
                ),
                String::from(
                    "/public/live2d/model/07airi/07airi_culture/07airi_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/07airi/07airi_fancostume/07airi_fancostume.model3.json",
                ),
                String::from(
                    "/public/live2d/model/07airi/07airi_maskcloth002/07airi_maskcloth002.model3.json",
                ),
                String::from("/public/live2d/model/07airi/07airi_normal/07airi_normal.model3.json"),
                String::from(
                    "/public/live2d/model/07airi/07airi_priestess/07airi_priestess.model3.json",
                ),
                String::from("/public/live2d/model/07airi/07airi_qt/07airi_qt.model3.json"),
                String::from(
                    "/public/live2d/model/07airi/07airi_qtnormal/07airi_qtnormal.model3.json",
                ),
                String::from("/public/live2d/model/07airi/07airi_unit/07airi_unit.model3.json"),
                String::from(
                    "/public/live2d/model/07airi/07airi_wedding/07airi_wedding.model3.json",
                ),
                String::from(
                    "/public/live2d/model/07airi/v2_07airi_casual/v2_07airi_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/07airi/v2_07airi_fancostume/v2_07airi_fancostume.model3.json",
                ),
                String::from(
                    "/public/live2d/model/07airi/v2_07airi_lesson/v2_07airi_lesson.model3.json",
                ),
                String::from(
                    "/public/live2d/model/07airi/v2_07airi_school01/v2_07airi_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/07airi/v2_07airi_unit/v2_07airi_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/08shizuku_archery/08shizuku_archery.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/08shizuku_cheerful/08shizuku_cheerful.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/08shizuku_cloth001/08shizuku_cloth001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/08shizuku_cloth002/08shizuku_cloth002.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/08shizuku_culture/08shizuku_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/08shizuku_fancostume/08shizuku_fancostume.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/08shizuku_normal/08shizuku_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/08shizuku_ponytail/08shizuku_ponytail.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/08shizuku_priestess/08shizuku_priestess.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/08shizuku_tuxedo/08shizuku_tuxedo.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/08shizuku_unit/08shizuku_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/v2_08shizuku_casual/v2_08shizuku_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/v2_08shizuku_casual_black/v2_08shizuku_casual_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/v2_08shizuku_fancostume/v2_08shizuku_fancostume.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/v2_08shizuku_fancostume_zoom/v2_08shizuku_fancostume_zoom.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/v2_08shizuku_lesson/v2_08shizuku_lesson.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/v2_08shizuku_ponytail/v2_08shizuku_ponytail.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/v2_08shizuku_school01/v2_08shizuku_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/08shizuku/v2_08shizuku_unit/v2_08shizuku_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/09kohane_culture/09kohane_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/09kohane_longnormal/09kohane_longnormal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/09kohane_longunit/09kohane_longunit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/09kohane_longunit_black/09kohane_longunit_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/09kohane_normal/09kohane_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/09kohane_sanisani/09kohane_sanisani.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/09kohane_sports/09kohane_sports.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/09kohane_sports02/09kohane_sports02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/09kohane_swim/09kohane_swim.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/09kohane_unit/09kohane_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/09kohane_unit_black/09kohane_unit_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/09kohane_wedding/09kohane_wedding.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/v2_09kohane_casual/v2_09kohane_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/v2_09kohane_school01/v2_09kohane_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/09kohane/v2_09kohane_unit/v2_09kohane_unit.model3.json",
                ),
                String::from("/public/live2d/model/10an/10an_black/10an_black.model3.json"),
                String::from("/public/live2d/model/10an/10an_culture/10an_culture.model3.json"),
                String::from("/public/live2d/model/10an/10an_jc/10an_jc.model3.json"),
                String::from("/public/live2d/model/10an/10an_normal/10an_normal.model3.json"),
                String::from("/public/live2d/model/10an/10an_sports/10an_sports.model3.json"),
                String::from("/public/live2d/model/10an/10an_sports02/10an_sports02.model3.json"),
                String::from("/public/live2d/model/10an/10an_unit/10an_unit.model3.json"),
                String::from("/public/live2d/model/10an/10an_vest/10an_vest.model3.json"),
                String::from("/public/live2d/model/10an/10an_wedding/10an_wedding.model3.json"),
                String::from("/public/live2d/model/10an/10an_yumeyume/10an_yumeyume.model3.json"),
                String::from("/public/live2d/model/10an/v2_10an_casual/v2_10an_casual.model3.json"),
                String::from(
                    "/public/live2d/model/10an/v2_10an_culture/v2_10an_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/10an/v2_10an_school01/v2_10an_school01.model3.json",
                ),
                String::from("/public/live2d/model/10an/v2_10an_unit/v2_10an_unit.model3.json"),
                String::from(
                    "/public/live2d/model/11akito/11akito_black/11akito_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/11akito/11akito_culture/11akito_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/11akito/11akito_normal/11akito_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/11akito/11akito_sports/11akito_sports.model3.json",
                ),
                String::from(
                    "/public/live2d/model/11akito/11akito_sports02/11akito_sports02.model3.json",
                ),
                String::from("/public/live2d/model/11akito/11akito_unit/11akito_unit.model3.json"),
                String::from("/public/live2d/model/11akito/11akito_vest/11akito_vest.model3.json"),
                String::from(
                    "/public/live2d/model/11akito/v2_11akito_casual/v2_11akito_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/11akito/v2_11akito_culture/v2_11akito_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/11akito/v2_11akito_school01/v2_11akito_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/11akito/v2_11akito_unit/v2_11akito_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/12touya/12touya_black/12touya_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/12touya/12touya_culture/12touya_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/12touya/12touya_normal/12touya_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/12touya/12touya_sports/12touya_sports.model3.json",
                ),
                String::from(
                    "/public/live2d/model/12touya/12touya_sports02/12touya_sports02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/12touya/12touya_tuxedo/12touya_tuxedo.model3.json",
                ),
                String::from("/public/live2d/model/12touya/12touya_unit/12touya_unit.model3.json"),
                String::from(
                    "/public/live2d/model/12touya/v2_12touya_casual/v2_12touya_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/12touya/v2_12touya_casual_zoom/v2_12touya_casual_zoom.model3.json",
                ),
                String::from(
                    "/public/live2d/model/12touya/v2_12touya_culture/v2_12touya_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/12touya/v2_12touya_school01/v2_12touya_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/12touya/v2_12touya_unit/v2_12touya_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/13tsukasa/13tsukasa_cloth001/13tsukasa_cloth001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/13tsukasa/13tsukasa_cloth01black/13tsukasa_cloth01black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/13tsukasa/13tsukasa_dc/13tsukasa_dc.model3.json",
                ),
                String::from(
                    "/public/live2d/model/13tsukasa/13tsukasa_liondance/13tsukasa_liondance.model3.json",
                ),
                String::from(
                    "/public/live2d/model/13tsukasa/13tsukasa_normal/13tsukasa_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/13tsukasa/13tsukasa_sports/13tsukasa_sports.model3.json",
                ),
                String::from(
                    "/public/live2d/model/13tsukasa/13tsukasa_sports02/13tsukasa_sports02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/13tsukasa/13tsukasa_unit/13tsukasa_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/13tsukasa/13tsukasa_yukata/13tsukasa_yukata.model3.json",
                ),
                String::from(
                    "/public/live2d/model/13tsukasa/v2_13tsukasa_casual/v2_13tsukasa_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/13tsukasa/v2_13tsukasa_casual_zoom/v2_13tsukasa_casual_zoom.model3.json",
                ),
                String::from(
                    "/public/live2d/model/13tsukasa/v2_13tsukasa_culture/v2_13tsukasa_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/13tsukasa/v2_13tsukasa_school01/v2_13tsukasa_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/13tsukasa/v2_13tsukasa_unit/v2_13tsukasa_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/14emu/14emu_cloth001/14emu_cloth001.model3.json",
                ),
                String::from("/public/live2d/model/14emu/14emu_culture/14emu_culture.model3.json"),
                String::from("/public/live2d/model/14emu/14emu_normal/14emu_normal.model3.json"),
                String::from("/public/live2d/model/14emu/14emu_sports/14emu_sports.model3.json"),
                String::from(
                    "/public/live2d/model/14emu/14emu_sports02/14emu_sports02.model3.json",
                ),
                String::from("/public/live2d/model/14emu/14emu_swim/14emu_swim.model3.json"),
                String::from("/public/live2d/model/14emu/14emu_unit/14emu_unit.model3.json"),
                String::from(
                    "/public/live2d/model/14emu/v2_14emu_casual/v2_14emu_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/14emu/v2_14emu_school01/v2_14emu_school01.model3.json",
                ),
                String::from("/public/live2d/model/14emu/v2_14emu_unit/v2_14emu_unit.model3.json"),
                String::from(
                    "/public/live2d/model/14emu/v2_14emu_unit_black/v2_14emu_unit_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/14emu/v2_14emu_unit_black_zoom/v2_14emu_unit_black_zoom.model3.json",
                ),
                String::from(
                    "/public/live2d/model/15nene/15nene_christmas/15nene_christmas.model3.json",
                ),
                String::from(
                    "/public/live2d/model/15nene/15nene_cloth001/15nene_cloth001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/15nene/15nene_culture/15nene_culture.model3.json",
                ),
                String::from("/public/live2d/model/15nene/15nene_jc/15nene_jc.model3.json"),
                String::from("/public/live2d/model/15nene/15nene_normal/15nene_normal.model3.json"),
                String::from("/public/live2d/model/15nene/15nene_sports/15nene_sports.model3.json"),
                String::from(
                    "/public/live2d/model/15nene/15nene_sports02/15nene_sports02.model3.json",
                ),
                String::from("/public/live2d/model/15nene/15nene_unit/15nene_unit.model3.json"),
                String::from(
                    "/public/live2d/model/15nene/v2_15nene_casual/v2_15nene_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/15nene/v2_15nene_culture/v2_15nene_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/15nene/v2_15nene_school01/v2_15nene_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/15nene/v2_15nene_unit/v2_15nene_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/15nene/v2_15nene_unit_black/v2_15nene_unit_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/15nene/v2_15nene_wedding/v2_15nene_wedding.model3.json",
                ),
                String::from(
                    "/public/live2d/model/16rui/16rui_cloth001/16rui_cloth001.model3.json",
                ),
                String::from("/public/live2d/model/16rui/16rui_culture/16rui_culture.model3.json"),
                String::from("/public/live2d/model/16rui/16rui_dc/16rui_dc.model3.json"),
                String::from("/public/live2d/model/16rui/16rui_normal/16rui_normal.model3.json"),
                String::from("/public/live2d/model/16rui/16rui_sports/16rui_sports.model3.json"),
                String::from(
                    "/public/live2d/model/16rui/16rui_sports02/16rui_sports02.model3.json",
                ),
                String::from("/public/live2d/model/16rui/16rui_unit/16rui_unit.model3.json"),
                String::from("/public/live2d/model/16rui/16rui_vest/16rui_vest.model3.json"),
                String::from("/public/live2d/model/16rui/16rui_yukata/16rui_yukata.model3.json"),
                String::from(
                    "/public/live2d/model/16rui/v2_16rui_casual/v2_16rui_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/16rui/v2_16rui_culture/v2_16rui_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/16rui/v2_16rui_school01/v2_16rui_school01.model3.json",
                ),
                String::from("/public/live2d/model/16rui/v2_16rui_unit/v2_16rui_unit.model3.json"),
                String::from(
                    "/public/live2d/model/16rui/v2_16rui_unit_black/v2_16rui_unit_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/17kanade/17kanade_april001/17kanade_april001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/17kanade/17kanade_black/17kanade_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/17kanade/17kanade_cloth001/17kanade_cloth001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/17kanade/17kanade_normal/17kanade_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/17kanade/17kanade_unit/17kanade_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/17kanade/17kanade_wedding/17kanade_wedding.model3.json",
                ),
                String::from(
                    "/public/live2d/model/17kanade/17kanade_yumeyume/17kanade_yumeyume.model3.json",
                ),
                String::from(
                    "/public/live2d/model/17kanade/v2_17kanade_casual/v2_17kanade_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/17kanade/v2_17kanade_casualblack/v2_17kanade_casualblack.model3.json",
                ),
                String::from(
                    "/public/live2d/model/17kanade/v2_17kanade_school02/v2_17kanade_school02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/17kanade/v2_17kanade_unit/v2_17kanade_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/18mafuyu_archery/18mafuyu_archery.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/18mafuyu_black/18mafuyu_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/18mafuyu_cloth001/18mafuyu_cloth001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/18mafuyu_culture/18mafuyu_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/18mafuyu_darkcloth001/18mafuyu_darkcloth001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/18mafuyu_darkunit/18mafuyu_darkunit.model3.json",
                ),
                String::from("/public/live2d/model/18mafuyu/18mafuyu_jc/18mafuyu_jc.model3.json"),
                String::from(
                    "/public/live2d/model/18mafuyu/18mafuyu_normal(1)/18mafuyu_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/18mafuyu_normal/18mafuyu_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/18mafuyu_sanisani/18mafuyu_sanisani.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/18mafuyu_sports/18mafuyu_sports.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/18mafuyu_sports02/18mafuyu_sports02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/18mafuyu_unit/18mafuyu_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/v2_18mafuyu_casual/v2_18mafuyu_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/v2_18mafuyu_darkunit/v2_18mafuyu_darkunit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/v2_18mafuyu_school01/v2_18mafuyu_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/v2_18mafuyu_school01_zoom/v2_18mafuyu_school01_zoom.model3.json",
                ),
                String::from(
                    "/public/live2d/model/18mafuyu/v2_18mafuyu_unit/v2_18mafuyu_unit.model3.json",
                ),
                String::from("/public/live2d/model/19ena/19ena_black/19ena_black.model3.json"),
                String::from(
                    "/public/live2d/model/19ena/19ena_cloth001/19ena_cloth001.model3.json",
                ),
                String::from("/public/live2d/model/19ena/19ena_jc/19ena_jc.model3.json"),
                String::from("/public/live2d/model/19ena/19ena_normal/19ena_normal.model3.json"),
                String::from("/public/live2d/model/19ena/19ena_unit/19ena_unit.model3.json"),
                String::from("/public/live2d/model/19ena/19ena_yukata/19ena_yukata.model3.json"),
                String::from(
                    "/public/live2d/model/19ena/v2_19ena_casual/v2_19ena_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/19ena/v2_19ena_culture/v2_19ena_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/19ena/v2_19ena_school01/v2_19ena_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/19ena/v2_19ena_school02/v2_19ena_school02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/19ena/v2_19ena_school_back/v2_19ena_school_back.model3.json",
                ),
                String::from("/public/live2d/model/19ena/v2_19ena_unit/v2_19ena_unit.model3.json"),
                String::from(
                    "/public/live2d/model/20mizuki/20mizuki_black/20mizuki_black.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/20mizuki_cloth001/20mizuki_cloth001.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/20mizuki_culture/20mizuki_culture.model3.json",
                ),
                String::from("/public/live2d/model/20mizuki/20mizuki_jc/20mizuki_jc.model3.json"),
                String::from(
                    "/public/live2d/model/20mizuki/20mizuki_jccloth/20mizuki_jccloth.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/20mizuki_normal/20mizuki_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/20mizuki_parttime/20mizuki_parttime.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/20mizuki_sports/20mizuki_sports.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/20mizuki_sports02/20mizuki_sports02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/20mizuki_unit/20mizuki_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/v2_20mizuki_casual/v2_20mizuki_casual.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/v2_20mizuki_culture/v2_20mizuki_culture.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/v2_20mizuki_culture_02/v2_20mizuki_culture_02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/v2_20mizuki_culture_back/v2_20mizuki_culture_back.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/v2_20mizuki_school01/v2_20mizuki_school01.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/v2_20mizuki_school01_02/v2_20mizuki_school01_02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/v2_20mizuki_school_back/v2_20mizuki_school_back.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/v2_20mizuki_school_back02/v2_20mizuki_school_back02.model3.json",
                ),
                String::from(
                    "/public/live2d/model/20mizuki/v2_20mizuki_unit/v2_20mizuki_unit.model3.json",
                ),
                String::from(
                    "/public/live2d/model/21miku/21miku_april001/21miku_april001.model3.json",
                ),
                String::from("/public/live2d/model/21miku/21miku_band/21miku_band.model3.json"),
                String::from(
                    "/public/live2d/model/21miku/21miku_bandblack/21miku_bandblack.model3.json",
                ),
                String::from("/public/live2d/model/21miku/21miku_idol/21miku_idol.model3.json"),
                String::from(
                    "/public/live2d/model/21miku/21miku_idolblack/21miku_idolblack.model3.json",
                ),
                String::from("/public/live2d/model/21miku/21miku_night/21miku_night.model3.json"),
                String::from("/public/live2d/model/21miku/21miku_normal/21miku_normal.model3.json"),
                String::from(
                    "/public/live2d/model/21miku/21miku_normalblack/21miku_normalblack.model3.json",
                ),
                String::from("/public/live2d/model/21miku/21miku_street/21miku_street.model3.json"),
                String::from(
                    "/public/live2d/model/21miku/21miku_streetblack/21miku_streetblack.model3.json",
                ),
                String::from("/public/live2d/model/21miku/21miku_wonder/21miku_wonder.model3.json"),
                String::from(
                    "/public/live2d/model/21miku/v2_21miku_band/v2_21miku_band.model3.json",
                ),
                String::from(
                    "/public/live2d/model/21miku/v2_21miku_idol/v2_21miku_idol.model3.json",
                ),
                String::from(
                    "/public/live2d/model/21miku/v2_21miku_night/v2_21miku_night.model3.json",
                ),
                String::from(
                    "/public/live2d/model/21miku/v2_21miku_normal/v2_21miku_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/21miku/v2_21miku_street/v2_21miku_street.model3.json",
                ),
                String::from(
                    "/public/live2d/model/21miku/v2_21miku_wonder/v2_21miku_wonder.model3.json",
                ),
                String::from(
                    "/public/live2d/model/22rin/22rin_april001/22rin_april001.model3.json",
                ),
                String::from("/public/live2d/model/22rin/22rin_band/22rin_band.model3.json"),
                String::from("/public/live2d/model/22rin/22rin_idol/22rin_idol.model3.json"),
                String::from("/public/live2d/model/22rin/22rin_night/22rin_night.model3.json"),
                String::from("/public/live2d/model/22rin/22rin_normal/22rin_normal.model3.json"),
                String::from(
                    "/public/live2d/model/22rin/22rin_normalblack/22rin_normalblack.model3.json",
                ),
                String::from("/public/live2d/model/22rin/22rin_street/22rin_street.model3.json"),
                String::from("/public/live2d/model/22rin/22rin_wonder/22rin_wonder.model3.json"),
                String::from("/public/live2d/model/22rin/v2_22rin_band/v2_22rin_band.model3.json"),
                String::from("/public/live2d/model/22rin/v2_22rin_idol/v2_22rin_idol.model3.json"),
                String::from(
                    "/public/live2d/model/22rin/v2_22rin_night/v2_22rin_night.model3.json",
                ),
                String::from(
                    "/public/live2d/model/22rin/v2_22rin_normal/v2_22rin_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/22rin/v2_22rin_street/v2_22rin_street.model3.json",
                ),
                String::from(
                    "/public/live2d/model/22rin/v2_22rin_wonder/v2_22rin_wonder.model3.json",
                ),
                String::from(
                    "/public/live2d/model/23len/23len_april001/23len_april001.model3.json",
                ),
                String::from("/public/live2d/model/23len/23len_band/23len_band.model3.json"),
                String::from("/public/live2d/model/23len/23len_idol/23len_idol.model3.json"),
                String::from("/public/live2d/model/23len/23len_night/23len_night.model3.json"),
                String::from("/public/live2d/model/23len/23len_normal/23len_normal.model3.json"),
                String::from("/public/live2d/model/23len/23len_street/23len_street.model3.json"),
                String::from("/public/live2d/model/23len/23len_wonder/23len_wonder.model3.json"),
                String::from("/public/live2d/model/23len/v2_23len_band/v2_23len_band.model3.json"),
                String::from("/public/live2d/model/23len/v2_23len_idol/v2_23len_idol.model3.json"),
                String::from(
                    "/public/live2d/model/23len/v2_23len_night/v2_23len_night.model3.json",
                ),
                String::from(
                    "/public/live2d/model/23len/v2_23len_normal/v2_23len_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/23len/v2_23len_street/v2_23len_street.model3.json",
                ),
                String::from(
                    "/public/live2d/model/23len/v2_23len_wonder/v2_23len_wonder.model3.json",
                ),
                String::from(
                    "/public/live2d/model/24luka/24luka_april001/24luka_april001.model3.json",
                ),
                String::from("/public/live2d/model/24luka/24luka_band/24luka_band.model3.json"),
                String::from("/public/live2d/model/24luka/24luka_idol/24luka_idol.model3.json"),
                String::from("/public/live2d/model/24luka/24luka_night/24luka_night.model3.json"),
                String::from("/public/live2d/model/24luka/24luka_normal/24luka_normal.model3.json"),
                String::from("/public/live2d/model/24luka/24luka_street/24luka_street.model3.json"),
                String::from("/public/live2d/model/24luka/24luka_wonder/24luka_wonder.model3.json"),
                String::from(
                    "/public/live2d/model/24luka/v2_24luka_band/v2_24luka_band.model3.json",
                ),
                String::from(
                    "/public/live2d/model/24luka/v2_24luka_idol/v2_24luka_idol.model3.json",
                ),
                String::from(
                    "/public/live2d/model/24luka/v2_24luka_night/v2_24luka_night.model3.json",
                ),
                String::from(
                    "/public/live2d/model/24luka/v2_24luka_normal/v2_24luka_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/24luka/v2_24luka_street/v2_24luka_street.model3.json",
                ),
                String::from(
                    "/public/live2d/model/24luka/v2_24luka_wonder/v2_24luka_wonder.model3.json",
                ),
                String::from(
                    "/public/live2d/model/25meiko/25meiko_april001/25meiko_april001.model3.json",
                ),
                String::from("/public/live2d/model/25meiko/25meiko_band/25meiko_band.model3.json"),
                String::from("/public/live2d/model/25meiko/25meiko_idol/25meiko_idol.model3.json"),
                String::from(
                    "/public/live2d/model/25meiko/25meiko_night/25meiko_night.model3.json",
                ),
                String::from(
                    "/public/live2d/model/25meiko/25meiko_normal/25meiko_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/25meiko/25meiko_street/25meiko_street.model3.json",
                ),
                String::from(
                    "/public/live2d/model/25meiko/25meiko_wonder/25meiko_wonder.model3.json",
                ),
                String::from(
                    "/public/live2d/model/25meiko/v2_25meiko_band/v2_25meiko_band.model3.json",
                ),
                String::from(
                    "/public/live2d/model/25meiko/v2_25meiko_idol/v2_25meiko_idol.model3.json",
                ),
                String::from(
                    "/public/live2d/model/25meiko/v2_25meiko_night/v2_25meiko_night.model3.json",
                ),
                String::from(
                    "/public/live2d/model/25meiko/v2_25meiko_normal/v2_25meiko_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/25meiko/v2_25meiko_street/v2_25meiko_street.model3.json",
                ),
                String::from(
                    "/public/live2d/model/25meiko/v2_25meiko_wonder/v2_25meiko_wonder.model3.json",
                ),
                String::from(
                    "/public/live2d/model/25meiko/v2_25meiko_wonderblack/v2_25meiko_wonderblack.model3.json",
                ),
                String::from(
                    "/public/live2d/model/26kaito/26kaito_april001/26kaito_april001.model3.json",
                ),
                String::from("/public/live2d/model/26kaito/26kaito_band/26kaito_band.model3.json"),
                String::from("/public/live2d/model/26kaito/26kaito_idol/26kaito_idol.model3.json"),
                String::from(
                    "/public/live2d/model/26kaito/26kaito_idolblack/26kaito_idolblack.model3.json",
                ),
                String::from(
                    "/public/live2d/model/26kaito/26kaito_night/26kaito_night.model3.json",
                ),
                String::from(
                    "/public/live2d/model/26kaito/26kaito_normal/26kaito_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/26kaito/26kaito_normalblack/26kaito_normalblack.model3.json",
                ),
                String::from(
                    "/public/live2d/model/26kaito/26kaito_street/26kaito_street.model3.json",
                ),
                String::from(
                    "/public/live2d/model/26kaito/26kaito_wonder/26kaito_wonder.model3.json",
                ),
                String::from(
                    "/public/live2d/model/26kaito/v2_26kaito_band/v2_26kaito_band.model3.json",
                ),
                String::from(
                    "/public/live2d/model/26kaito/v2_26kaito_idol/v2_26kaito_idol.model3.json",
                ),
                String::from(
                    "/public/live2d/model/26kaito/v2_26kaito_night/v2_26kaito_night.model3.json",
                ),
                String::from(
                    "/public/live2d/model/26kaito/v2_26kaito_normal/v2_26kaito_normal.model3.json",
                ),
                String::from(
                    "/public/live2d/model/26kaito/v2_26kaito_normalblack/v2_26kaito_normalblack.model3.json",
                ),
                String::from(
                    "/public/live2d/model/26kaito/v2_26kaito_street/v2_26kaito_street.model3.json",
                ),
                String::from(
                    "/public/live2d/model/26kaito/v2_26kaito_wonder/v2_26kaito_wonder.model3.json",
                ),
            ],
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct BuildMotionData {
    pub expressions: Vec<String>,
    pub motions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MotionEntry {
    #[serde(rename = "FadeInTime")]
    fade_in_time: f32,

    #[serde(rename = "FadeOutTime")]
    fade_out_time: f32,

    #[serde(rename = "File")]
    pub file: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileReferences {
    #[serde(rename = "Moc")]
    moc: String,

    #[serde(rename = "Motions")]
    pub motions: IndexMap<String, Vec<MotionEntry>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Model3Root {
    #[serde(rename = "FileReferences")]
    pub file_references: FileReferences,
}

pub fn build_character_map() -> HashMap<String, &'static str> {
    let mut map = HashMap::new();

    map.insert("ichika".to_string(), "01_ichika");
    map.insert("saki".to_string(), "02_saki");
    map.insert("honami".to_string(), "03_honami");
    map.insert("shiho".to_string(), "04_shiho");
    map.insert("minori".to_string(), "05_minori");
    map.insert("haruka".to_string(), "06_haruka");
    map.insert("airi".to_string(), "07_airi");
    map.insert("shizuku".to_string(), "08_shizuku");
    map.insert("kohane".to_string(), "09_kohane");
    map.insert("an".to_string(), "10_an");
    map.insert("akito".to_string(), "11_akito");
    map.insert("touya".to_string(), "12_touya");
    map.insert("tsukasa".to_string(), "13_tsukasa");
    map.insert("emu".to_string(), "14_emu");
    map.insert("nene".to_string(), "15_nene");
    map.insert("rui".to_string(), "16_rui");
    map.insert("kanade".to_string(), "17_kanade");
    map.insert("mafuyu".to_string(), "18_mafuyu");
    map.insert("ena".to_string(), "19_ena");
    map.insert("mizuki".to_string(), "20_mizuki");
    map.insert("miku".to_string(), "21_miku");
    map.insert("rin".to_string(), "22_rin");
    map.insert("len".to_string(), "23_len");
    map.insert("luka".to_string(), "24_luka");
    map.insert("meiko".to_string(), "25_meiko");
    map.insert("kaito".to_string(), "26_kaito");

    map
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ABInfoRoot {
    pub version: String,
    pub os: String,
    pub bundles: HashMap<String, ABInfoBundle>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ABInfoBundle {
    pub bundle_name: String,
    pub cache_file_name: String,
    pub cache_directory_name: String,
    pub hash: String,
    pub category: String,
    pub crc: u32,
    pub file_size: u32,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub paths: Vec<String>,
    pub is_builtin: bool,
}
