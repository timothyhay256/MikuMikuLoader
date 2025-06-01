use serde::Deserialize;

// Contains all relevant data that makes up a scenario. Used to adapt between Sekai Stories and AssetBundles.

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapter {
    pub name: String,
    pub scenario_id: String,
    pub first_bgm: String,
    pub mv_id: String,

    pub first_background: String,
    pub first_character_layout_mode: i32,

    pub speed: f32,
    pub font_size: f32,

    pub when_finish_close_window: bool,
    pub require_play_effect: bool,

    pub appear_characters: Vec<ScenarioAdapterAppearCharacters>,
    pub talk_data: Vec<ScenarioAdapterTalkData>,
    pub character_layout: Vec<ScenarioAdapterCharacterLayout>,
}

impl Default for ScenarioAdapter {
    fn default() -> Self {
        ScenarioAdapter {
            name: "Mod".to_string(),
            scenario_id: "event_129_01".to_string(), // Whip event
            first_bgm: "bgm00000".to_string(),
            mv_id: "".to_string(),
            first_background: "bg_c001101".to_string(),
            first_character_layout_mode: 0,
            speed: 0.0,
            font_size: 0.0,
            when_finish_close_window: true,
            require_play_effect: false,
            appear_characters: Vec::new(),
            talk_data: Vec::new(),
            character_layout: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapterAppearCharacters {
    // SEKAI Stories needs to fill this, no default is provided
    pub character_2d_id: i32,
    pub character_costume: String,
}

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapterTalkData {
    pub character_2d_id: i32,
    pub display_name: String,
    pub text: String,
    pub talk_tention: i32,
    pub lib_sync: i32,

    pub motion: Vec<ScenarioAdapterTalkDataMotion>,
    pub voice: Vec<ScenarioAdapterTalkDataVoices>,
    pub special_effects: Vec<ScenarioAdapterSpecialEffect>,
    pub sound: Vec<ScenarioAdapterSoundData>,

    pub needed_bundles: ScenarioAdapterNeededBundles,
}

impl Default for ScenarioAdapterTalkData {
    fn default() -> Self {
        ScenarioAdapterTalkData {
            character_2d_id: 286,
            display_name: "Kohane".to_string(),
            text: "Scene text".to_string(),
            talk_tention: 0,
            lib_sync: 1,
            motion: Vec::new(),
            voice: Vec::new(),
            special_effects: Vec::new(),
            sound: Vec::new(),
            needed_bundles: ScenarioAdapterNeededBundles::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapterTalkDataMotion {
    pub motion_name: String,
    pub facial_name: String,
}

impl Default for ScenarioAdapterTalkDataMotion {
    fn default() -> Self {
        ScenarioAdapterTalkDataMotion {
            motion_name: "w-normal15-tilthead".to_string(),
            facial_name: "face_smallmouth_01".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapterTalkDataVoices {
    pub voice_id: String,
    pub volume: f32,
}

impl Default for ScenarioAdapterTalkDataVoices {
    fn default() -> Self {
        ScenarioAdapterTalkDataVoices {
            voice_id: "voice_ev_street_17_01_01_09".to_string(), // TODO: Eventually load custom voices
            volume: 0.0, // TODO: Replace with 1 once voices are working
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapterCharacterLayout {
    pub character_2d_id: i32,
    pub layout_type: i32,
    pub depth_type: i32,
    pub move_speed_type: i32,

    pub side_from: i32,
    pub side_from_offset_x: f32,
    pub side_to: i32,
    pub side_to_offset_x: f32,

    pub costume_type: String,

    pub motion: ScenarioAdapterTalkDataMotion,
}

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapterSpecialEffect {
    pub effect_type: i32,
    pub duration: f32,
    pub int_val: i32,

    pub string: String,
    pub string_sub: String,
}

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapterSoundData {
    pub play_mode: i32,

    pub bgm_string: String,
    pub se_string: String,
    pub se_bundle_name: String,

    pub volume: f32,
    pub duration: f32,
}

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapterNeededBundles {
    pub bundle_names: Vec<String>,
    pub sound_bundle_names: Vec<String>,
}

impl Default for ScenarioAdapterNeededBundles {
    fn default() -> Self {
        ScenarioAdapterNeededBundles {
            bundle_names: Vec::new(),
            sound_bundle_names: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapterCharacterLayoutMode {
    pub layouts: Vec<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CustomStory {
    pub file_name: String,
    pub data: Vec<CustomStoryScene>,
}

#[derive(Debug, Deserialize)]
pub struct CustomStoryScene {
    pub index: i64,
    pub data: SekaiStoriesScene,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SekaiStoriesScene {
    pub last_modified: String,
    pub background: String,
    pub text: SekaiStoriesSceneText,
    pub models: Vec<SekaiStoriesSceneModels>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SekaiStoriesSceneText {
    pub name_tag: String,
    pub dialogue: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SekaiStoriesSceneModels {
    pub from: String,
    pub character: String,
    pub modelName: String,
    pub modelTransform: SekaiStoriesSceneTransform,
    pub modelExpression: i32,
    pub modelPose: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SekaiStoriesSceneTransform {
    pub x: i32,
    pub y: i32,
    pub scale: f32,
}
