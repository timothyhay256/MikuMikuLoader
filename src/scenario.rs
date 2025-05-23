// Contains all relevant data that makes up a scenario. Used to adapt between Sekai Stories and AssetBundles.
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

pub struct ScenarioAdapterAppearCharacters {
    character_2d_id: i32,
    character_costume: String,
}

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

pub struct ScenarioAdapterTalkDataMotion {
    motion_name: String,
    facial_name: String,
}

pub struct ScenarioAdapterTalkDataVoices {
    voice_id: String,
    volume: f32,
}

pub struct ScenarioAdapterCharacterLayout {
    character_2d_id: i32,
    layout_type: i32,
    depth_type: i32,
    move_speed_type: i32,

    side_from: i32,
    side_from_offset_x: f32,
    #[allow(dead_code)]
    side_to: i32,
    side_to_offset_x: f32,

    costume_type: String,

    motion: ScenarioAdapterTalkDataMotion,
}

pub struct ScenarioAdapterSpecialEffect {
    effect_type: i32,
    duration: f32,
    int_val: i32,

    string: String,
    string_sub: String,
}

pub struct ScenarioAdapterSoundData {
    play_mode: i32,

    bgm_string: String,
    se_string: String,
    se_bundle_name: String,

    volume: f32,
    duration: f32,
}

pub struct ScenarioAdapterNeededBundles {
    bundle_names: Vec<String>,
    sound_bundle_names: Vec<String>,
}

pub struct ScenarioAdapterCharacterLayoutMode {
    layouts: Vec<i32>,
}

#[allow(non_snake_case)] // Sekai Stories uses snake case.
pub struct SekaiStoriesScene {
    lastModified: String,
    background: String,
    text: SekaiStoriesSceneText,
    models: Vec<SekaiStoriesSceneModels>,
}

#[allow(non_snake_case)]
pub struct SekaiStoriesSceneText {
    nameTag: String,
    dialogue: String,
}

#[allow(non_snake_case)]
pub struct SekaiStoriesSceneModels {
    from: String,
    character: String,
    modelName: String,
    modelTransform: SekaiStoriesSceneTransform,
    modelExpression: i32,
    modelPose: i32,
}

#[allow(non_snake_case)]
pub struct SekaiStoriesSceneTransform {
    x: i32,
    y: i32,
    scale: f32,
}
