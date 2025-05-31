use serde::Deserialize;

// Contains all relevant data that makes up a scenario. Used to adapt between Sekai Stories and AssetBundles.

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapter {
    name: String,
    scenario_id: String,
    first_bgm: String,
    mv_id: String,

    first_background: String,
    first_character_layout_mode: i32,

    speed: f32,
    font_size: f32,

    when_finish_close_window: bool,
    require_play_effect: bool,

    appear_characters: Vec<ScenarioAdapterAppearCharacters>,
    talk_data: Vec<ScenarioAdapterTalkData>,
    character_layout: Vec<ScenarioAdapterCharacterLayout>,
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
    character_2d_id: i32,
    character_costume: String,
}

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapterTalkData {
    character_2d_id: i32,
    display_name: String,
    text: String,
    talk_tention: i32,
    lib_sync: i32,

    motion: Vec<ScenarioAdapterTalkDataMotion>,
    voice: Vec<ScenarioAdapterTalkDataVoices>,
    special_effects: Vec<ScenarioAdapterSpecialEffect>,
    sound: Vec<ScenarioAdapterSoundData>,

    needed_bundles: ScenarioAdapterNeededBundles,
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
    motion_name: String,
    facial_name: String,
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
    voice_id: String,
    volume: f32,
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
    character_2d_id: i32,
    layout_type: i32,
    depth_type: i32,
    move_speed_type: i32,

    side_from: i32,
    side_from_offset_x: f32,
    side_to: i32,
    side_to_offset_x: f32,

    costume_type: String,

    motion: ScenarioAdapterTalkDataMotion,
}

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapterSpecialEffect {
    effect_type: i32,
    duration: f32,
    int_val: i32,

    string: String,
    string_sub: String,
}

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapterSoundData {
    play_mode: i32,

    bgm_string: String,
    se_string: String,
    se_bundle_name: String,

    volume: f32,
    duration: f32,
}

#[derive(Debug, Deserialize)]
pub struct ScenarioAdapterNeededBundles {
    bundle_names: Vec<String>,
    sound_bundle_names: Vec<String>,
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
    layouts: Vec<i32>,
}

#[allow(non_snake_case)] // Sekai Stories uses snake case.
#[derive(Debug, Deserialize)]
pub struct SekaiStoriesScene {
    lastModified: String,
    background: String,
    text: SekaiStoriesSceneText,
    models: Vec<SekaiStoriesSceneModels>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct SekaiStoriesSceneText {
    nameTag: String,
    dialogue: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct SekaiStoriesSceneModels {
    from: String,
    character: String,
    modelName: String,
    modelTransform: SekaiStoriesSceneTransform,
    modelExpression: i32,
    modelPose: i32,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct SekaiStoriesSceneTransform {
    x: i32,
    y: i32,
    scale: f32,
}
