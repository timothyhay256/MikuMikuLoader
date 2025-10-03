use std::{
    collections::HashMap,
    ffi::CString,
    fs::{self, copy},
    path::{Path, PathBuf},
};

use anyhow::Result;
use log::{debug, error, info, warn};
use pyo3::{
    PyResult, Python,
    types::{PyAnyMethods, PyModule},
};
use pythonize::{depythonize, pythonize};
// use dict_derive::FromPyObject;
use serde::{Deserialize, Serialize};

use crate::{
    encrypt,
    mods::{ModData, ModType},
    notify_mml,
    utils::{self, Character2DS, Model3Root},
};

/// Contains all relevant UnityPy code for loading and exporting a typetree.
pub const PY_CODE: &str = include_str!("../python/story_to_assetbundle.py");

/// The path id to the scenario in the template we are using, which in this case is the whip_2024 scenario but stripped of anything but the bare min.
pub static SCENARIO_PATH_ID: i64 = 6343946530110770478;

/// Contains all relevant data that makes up a scenario.
/// Directly represents the typetree from UnityPy.
/// Uses camelCase to match the UnityPy typetree.
#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct Scenario {
    pub m_GameObject: UnityReference,
    pub m_Enabled: i32,
    pub m_Script: UnityReference,
    pub m_Name: String,

    #[serde(rename = "ScenarioId")]
    pub scenarioId: String,

    #[serde(rename = "AppearCharacters")]
    pub appearCharacters: Vec<ScenarioAppearCharacters>,

    /// Nearly always empty, but FromPyObject doesn't like Vec<()>
    #[serde(rename = "FirstLayout")]
    pub firstLayout: Vec<String>,

    #[serde(rename = "FirstBgm")]
    pub firstBgm: String,

    #[serde(rename = "EpisodeMusicVideoId")]
    pub episodeMusicVideoId: String,

    #[serde(rename = "FirstBackground")]
    pub firstBackground: String,

    #[serde(rename = "FirstAisacValue")]
    pub firstAisacValue: String,

    #[serde(rename = "FirstCharacterLayoutMode")]
    pub firstCharacterLayoutMode: i32,

    #[serde(rename = "Snippets")]
    pub snippets: Vec<ScenarioSnippet>,

    #[serde(rename = "TalkData")]
    pub talkData: Vec<ScenarioTalkData>,

    #[serde(rename = "LayoutData")]
    pub layoutData: Vec<ScenarioCharacterLayout>,

    #[serde(rename = "SpecialEffectData")]
    pub specialEffectData: Vec<ScenarioSpecialEffect>,

    #[serde(rename = "SoundData")]
    pub soundData: Vec<ScenarioSoundData>,

    #[serde(rename = "NeedBundleNames")]
    pub needBundleNames: Vec<String>,

    #[serde(rename = "IncludeSoundDataBundleNames")]
    pub includeSoundDataBundleNames: Vec<String>,

    #[serde(rename = "ScenarioSnippetCharacterLayoutModes")]
    pub scenarioSnippetCharacterLayoutModes: Vec<ScenarioSnippetLayoutMode>,
}

impl Default for Scenario {
    fn default() -> Self {
        Scenario {
            m_GameObject: UnityReference::default(),
            m_Enabled: 1,
            m_Script: UnityReference::default(),
            m_Name: "event_129_01".to_string(),
            scenarioId: "event_129_01".to_string(),
            appearCharacters: vec![ScenarioAppearCharacters::default()],
            firstLayout: Vec::new(),
            firstBgm: "bgm00000".to_string(),
            episodeMusicVideoId: "".to_string(),
            firstBackground: "bg_c001101".to_string(),
            firstAisacValue: "".to_string(),
            firstCharacterLayoutMode: 0,
            snippets: vec![ScenarioSnippet::default()],
            talkData: vec![ScenarioTalkData::default()],
            layoutData: vec![ScenarioCharacterLayout::default()],
            specialEffectData: vec![ScenarioSpecialEffect::default()],
            soundData: vec![ScenarioSoundData::default()],
            needBundleNames: vec!["scenario/background/bg_c001101".to_string()],
            includeSoundDataBundleNames: Vec::new(),
            scenarioSnippetCharacterLayoutModes: vec![ScenarioSnippetLayoutMode::default()],
        }
    }
}

impl Scenario {
    pub fn generate_story_assetbundle(&mut self, payload: &CustomStory) {
        let mod_name = payload.modpack_name.clone();

        // Store all characters and their expressions while looping through models to be used later
        let mut character_expressions: Option<HashMap<String, CharacterData>> = None;

        // Load character2ds
        let character2ds_file = fs::File::open("assets/character2ds.json").expect("Could not read assets/character2ds.json! Please remove the assets folder and try again to redownload assets.");

        let character2ds: Vec<Character2DS> = serde_json::from_reader(character2ds_file).expect(
            "character2ds.json is not formatted properly! Check if MikuMikuLoader is out of date.",
        );

        let character2ds_map: HashMap<String, Character2DS> = character2ds
            .into_iter()
            .filter_map(|c| c.asset_name.clone().map(|name| (name, c)))
            .collect();

        // Push the first background
        let bkg_name = Path::new(&payload.data[0].data.background.clone().to_string())
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_owned();

        debug!("Pushing first background");
        self.firstBackground = bkg_name.clone();

        self.needBundleNames
            .push(format!("scenario/background/{bkg_name}"));

        // Push special effect data with story name and (?) MikuMikuLoader
        self.specialEffectData.push(ScenarioSpecialEffect {
            effectType: 8,
            stringVal: "Created with MikuMikuLoader".to_owned(),
            stringValSub: "".to_owned(),
            duration: 0.0,
            intVal: 0,
        });

        self.specialEffectData.push(ScenarioSpecialEffect {
            effectType: 8,
            stringVal: mod_name.to_owned(),
            stringValSub: "".to_owned(),
            duration: 0.0,
            intVal: 0,
        });

        self.specialEffectData.push(ScenarioSpecialEffect {
            effectType: 4,
            stringVal: "".to_owned(),
            stringValSub: "".to_owned(),
            duration: 1.0,
            intVal: 0,
        });

        // Push necessary ScenarioSnippet for above special effects

        // "MikuMikuLoader" special effect
        self.snippets.push(ScenarioSnippet {
            index: 0,
            action: 6,
            progressBehavior: 1,
            referenceIndex: 0,
            delay: 0.0,
        });

        // Mod name special effect
        self.snippets.push(ScenarioSnippet {
            index: 1,
            action: 6,
            progressBehavior: 1,
            referenceIndex: 1,
            delay: 0.0,
        });

        // Clear special effects
        self.snippets.push(ScenarioSnippet {
            index: 2,
            action: 7,
            progressBehavior: 1,
            referenceIndex: 0,
            delay: 0.0,
        });

        // Have the character appear TODO: Multiple character support
        self.snippets.push(ScenarioSnippet {
            index: 3,
            action: 2,
            progressBehavior: 1,
            referenceIndex: 0,
            delay: 2.0,
        });

        // Loop through all the scenes to push the relevant data
        for (index, scene) in payload.data.iter().enumerate() {
            let initial_scene = { index == 0 };
            // Populate appear_characters, and use the grabbed id to populate talk_data at the same time
            for model in &scene.data.models {
                if model.from == "sekai" {
                    // Extract id by extracting model name from costume
                    let asset_prefix = if let Some(pos) = model.model_name.rfind('_') {
                        let before_last = &model.model_name[..pos];

                        if before_last.contains('_') {
                            // multiple underscores, return everything before last underscore
                            before_last
                        } else if before_last.contains("v2") {
                            // contains v2 prefix, return whole thing
                            &model.model_name
                        } else {
                            // one underscore, no v2 prefix, return part before first underscore
                            match model.model_name.find('_') {
                                Some(first_pos) => &model.model_name[..first_pos],
                                None => &model.model_name,
                            }
                        }
                    } else {
                        // No underscores at all
                        &model.model_name
                    };

                    debug!("getting character_id from prefix: {asset_prefix}");

                    let character_id = character2ds_map
                        .get(asset_prefix)
                        .expect("missing character")
                        .id;

                    debug!("Setting character_id to {character_id}");

                    // Use BuildMotionData to determine the expression and pose, since SEKAI-Stories uses an index in the serialized JSON
                    character_expressions.get_or_insert_with(HashMap::new).insert(model.character.clone(), {
                    let char_map = utils::build_character_map();

                    if let Some(full_id) = char_map.get(&model.character) {

                        // Get pose and expression name from SEKAI-Stories index
                        let character_motions_path =
                            format!("assets/public/live2d/model/{}/{}/{}.model3.json", full_id.replace("_", ""), model.model_name, model.model_name);

                        debug!("Trying to read build motion data from {character_motions_path}");

                        let character_motions_file = fs::File::open(&character_motions_path).unwrap_or_else(|_| panic!("Could not read {}! Please remove the assets folder and try again to redownload assets.", &character_motions_path));

                        let character_motions: Model3Root = serde_json::from_reader(
                            character_motions_file,
                        ).unwrap();

                        // Collect just the keys
                        let mut character_motions: Vec<&String> = character_motions.file_references.motions.keys().collect();

                        // SEKAI-Stories has a bug(?) where v2_20mizuki_casual has face_sleepy_03, despite not being referanced in any model files for Mizuki, so it has to be inserted.
                        let missing_motion = &"face_sleepy_03".to_string();
                        if model.model_name == "v2_20mizuki_casual" {
                            debug!("Inserting face_sleepy_03 to account for SEKAI-Stories");

                            let insert_index = character_motions.iter().position(|&p| p == "face_sleepy_02").unwrap();

                            character_motions.insert(insert_index + 1, missing_motion);
                        }

                        // Grabs pose name
                        let result_pose = match character_motions.get(model.model_pose as usize) {
                            Some(result_pose) => *result_pose,
                            None => {
                                warn!("Could not find any matching result_pose inside {character_motions_path}, using w-adult-blushed01!");
                                &"w-adult-blushed01".to_string()
                            }
                        };

                        // Get index of first key containing "face_", as this is how SEKAI-Stories does its indexing

                        let result_expression = match character_motions.iter().position(|k| k.contains("face_")) {
                            Some(idx) => match character_motions.get(idx + (model.model_expression - 1) as usize) {
                                Some(result_expression) => *result_expression,
                                None => {
                                    warn!("Could not find any matching result_expression inside {character_motions_path}, using face_cry_01!");
                                    &"face_cry_01".to_string()
                                }
                            },
                            None => {
                                warn!("{character_motions_path}, contains no face_*! This should never happen, using face_cry_01!");
                                &"face_cry_01".to_string()
                            }
                        };

                        let char_data = CharacterData {
                            id: character_id,
                            motion_name: result_pose.to_owned(),
                            facial_name: result_expression.to_owned(),
                            costume_type: model.model_name.clone(),
                        };

                        debug!("Inserting the following CharacterData: {char_data:?}");
                        char_data
                    } else {
                        error!(
                            "Could not find character {}! Please create a bug report. The expression will be replaced with face_cry_01, and the pose will be replaced with w-cute-glad01",
                            "face_cry_01"
                        );

                        CharacterData {
                            id: 286,
                            motion_name: "w-cute-glad01".to_string(),
                            facial_name: "face_cry_01".to_string(),
                            costume_type: "v2_09kohane_casual".to_string(),
                        }
                    }
                });

                    let character_to_push = crate::scenario::ScenarioAppearCharacters {
                        character2dId: character_id,
                        costumeType: model.model_name.clone(),
                    };

                    if !self.appearCharacters.contains(&character_to_push) {
                        debug!("Pushing appearCharacters: {character_to_push:?}");
                        self // Push character to appear_characters
                            .appearCharacters
                            .push(character_to_push);
                    }
                } else {
                    error!(
                        "Only characters from Project Sekai are currently supported! Skipping {}",
                        model.character
                    );
                    notify_mml(
                        "Found a non-Sekai character in cust story. Only characters from Project Sekai are currently supported!",
                    );
                }
            }

            // Apply text and stories
            let character_name = &scene.data.text.name_tag.to_lowercase();

            match character_expressions {
                Some(ref character_expressions) => {
                    match character_expressions.get(character_name) {
                        Some(character) => {
                            self.talkData.push(ScenarioTalkData {
                                // Push character to talk_data (which will have other fields filled later)
                                talkCharacters: vec![TalkCharacter {
                                    character2dId: character.id,
                                }],
                                windowDisplayName: capitalize(character_name),
                                body: scene.data.text.dialogue.clone(),
                                motions: {
                                    if initial_scene {
                                        Vec::new()
                                    } else {
                                        vec![TalkMotion {
                                            motionName: character.motion_name.clone(),
                                            facialName: character.facial_name.clone(),
                                            character2dId: character.id,
                                            ..Default::default()
                                        }]
                                    }
                                },
                                voices: Vec::new(),
                                whenFinishCloseWindow: {
                                    // TODO: Make configurable
                                    if index == payload.data.len() { 1 } else { 0 }
                                },
                                ..Default::default()
                            });

                            let scenario_char_layout = ScenarioCharacterLayout {
                                r#type: if initial_scene { 2 } else { 0 },
                                sideFrom: if initial_scene { 4 } else { 3 },
                                sideFromOffsetX: 0.0,
                                sideTo: if initial_scene { 4 } else { 3 },
                                sideToOffsetX: 0.0,
                                depthType: 0,
                                character2dId: character.id,
                                costumeType: if initial_scene {
                                    character.costume_type.clone()
                                } else {
                                    "".to_owned()
                                },
                                motionName: character.motion_name.clone(),
                                facialName: character.facial_name.clone(),
                                moveSpeedType: 0,
                            };

                            self.layoutData.push(scenario_char_layout);

                            // If it's the last scene, we need to push an empty layoutdata. Not sure why.
                            if index == payload.data.len() - 1 {
                                debug!(
                                    "Pushing final needed empty ScenarioSnippetCharacterLayout!"
                                );

                                self.layoutData.push(ScenarioCharacterLayout {
                                    r#type: 3,
                                    sideFrom: 4,
                                    sideFromOffsetX: 0.0,
                                    sideTo: 4,
                                    sideToOffsetX: 0.0,
                                    depthType: 0,
                                    character2dId: character.id,
                                    costumeType: "".to_owned(),
                                    motionName: "".to_owned(),
                                    facialName: "".to_owned(),
                                    moveSpeedType: 0,
                                })
                            }
                            // match self.layoutData.contains(&scenario_char_layout) {
                            //     true => {}
                            //     false => self.layoutData.push(scenario_char_layout),
                            // };

                            // Push an ScenarioSnippet for our dialogue
                            self.snippets.push(ScenarioSnippet {
                                index: self.snippets.len() as i32,
                                action: 1,
                                progressBehavior: 1,
                                referenceIndex: index as i32,
                                delay: 2.0,
                            });
                        }
                        None => {
                            error!(
                                "Could not find {character_name} in {character_expressions:?}! Please file a bug report. Default character will be used."
                            );
                            self.talkData.push(ScenarioTalkData::default())
                        }
                    }
                }
                None => {
                    error!(
                        "No characters were correctly initialized in the last step. Tried to find {character_name} but character_expressions is None"
                    );
                }
            }
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// References to Unity objects (script, game object, etc.)
#[derive(Debug, Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct UnityReference {
    pub m_FileID: i32,
    pub m_PathID: i64,
}

#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[allow(non_snake_case)]
pub struct ScenarioAppearCharacters {
    #[serde(rename = "Character2dId")]
    pub character2dId: i32,

    #[serde(rename = "CostumeType")]
    pub costumeType: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct ScenarioSnippet {
    #[serde(rename = "Index")]
    pub index: i32,

    #[serde(rename = "Action")]
    pub action: i32,

    #[serde(rename = "ProgressBehavior")]
    pub progressBehavior: i32,

    #[serde(rename = "ReferenceIndex")]
    pub referenceIndex: i32,

    #[serde(rename = "Delay")]
    pub delay: f32,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ScenarioTalkData {
    #[serde(rename = "TalkCharacters")]
    pub talkCharacters: Vec<TalkCharacter>,

    #[serde(rename = "WindowDisplayName")]
    pub windowDisplayName: String,

    #[serde(rename = "Body")]
    pub body: String,

    #[serde(rename = "TalkTention")]
    pub talkTention: i32,

    #[serde(rename = "LipSync")]
    pub lipSync: i32,

    #[serde(rename = "MotionChangeFrom")]
    pub motionChangeFrom: i32,

    #[serde(rename = "Motions")]
    pub motions: Vec<TalkMotion>,

    #[serde(rename = "Voices")]
    pub voices: Vec<TalkVoice>,

    #[serde(rename = "Speed")]
    pub speed: f32,

    #[serde(rename = "FontSize")]
    pub fontSize: i32,

    #[serde(rename = "WhenFinishCloseWindow")]
    pub whenFinishCloseWindow: i32,

    #[serde(rename = "RequirePlayEffect")]
    pub requirePlayEffect: i32,

    #[serde(rename = "EffectReferenceIdx")]
    pub effectReferenceIdx: i32,

    #[serde(rename = "RequirePlaySound")]
    pub requirePlaySound: i32,

    #[serde(rename = "SoundReferenceIdx")]
    pub soundReferenceIdx: i32,

    #[serde(rename = "TargetValueScale")]
    pub targetValueScale: f32,
}

impl Default for ScenarioTalkData {
    fn default() -> Self {
        ScenarioTalkData {
            talkCharacters: vec![TalkCharacter { character2dId: 286 }],
            windowDisplayName: "Kohane".to_string(),
            body: "Default body".to_string(),
            talkTention: 0,
            lipSync: 1,
            motionChangeFrom: 1,
            motions: vec![TalkMotion::default()],
            voices: vec![TalkVoice::default()],
            speed: 0.0,
            fontSize: 0,
            whenFinishCloseWindow: 1,
            requirePlayEffect: 0,
            effectReferenceIdx: 0,
            requirePlaySound: 0,
            soundReferenceIdx: 0,
            targetValueScale: 0.0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct TalkCharacter {
    #[serde(rename = "Character2dId")]
    pub character2dId: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct TalkMotion {
    #[serde(rename = "Character2dId")]
    pub character2dId: i32,

    #[serde(rename = "MotionName")]
    pub motionName: String,

    #[serde(rename = "FacialName")]
    pub facialName: String,

    #[serde(rename = "TimingSyncValue")]
    pub timingSyncValue: f32,
}

impl Default for TalkMotion {
    fn default() -> Self {
        TalkMotion {
            character2dId: 286,
            motionName: "w-normal15-shy".to_string(),
            facialName: "face_smallmouth_01".to_string(),
            timingSyncValue: 0.0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct TalkVoice {
    #[serde(rename = "Character2dId")]
    pub character2dId: i32,

    #[serde(rename = "VoiceId")]
    pub voiceId: String,

    #[serde(rename = "Volume")]
    pub volume: f32,
}

impl Default for TalkVoice {
    fn default() -> Self {
        TalkVoice {
            character2dId: 286,
            voiceId: "voice_ev_street_17_01_01_09".to_string(),
            volume: 1.0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[allow(non_snake_case)]
pub struct ScenarioCharacterLayout {
    #[serde(rename = "Type")]
    pub r#type: i32,

    #[serde(rename = "SideFrom")]
    pub sideFrom: i32,

    #[serde(rename = "SideFromOffsetX")]
    pub sideFromOffsetX: f32,

    #[serde(rename = "SideTo")]
    pub sideTo: i32,

    #[serde(rename = "SideToOffsetX")]
    pub sideToOffsetX: f32,

    #[serde(rename = "DepthType")]
    pub depthType: i32,

    #[serde(rename = "Character2dId")]
    pub character2dId: i32,

    #[serde(rename = "CostumeType")]
    pub costumeType: String,

    #[serde(rename = "MotionName")]
    pub motionName: String,

    #[serde(rename = "FacialName")]
    pub facialName: String,

    #[serde(rename = "MoveSpeedType")]
    pub moveSpeedType: i32,
}

impl Default for ScenarioCharacterLayout {
    fn default() -> Self {
        ScenarioCharacterLayout {
            r#type: 2,
            sideFrom: 4,
            sideFromOffsetX: 0.0,
            sideTo: 4,
            sideToOffsetX: 0.0,
            depthType: 0,
            character2dId: 286,
            costumeType: "v2_09kohane_casual".to_string(),
            motionName: "w-normal15-tilthead".to_string(),
            facialName: "face_smallmouth_01".to_string(),
            moveSpeedType: 0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ScenarioSpecialEffect {
    #[serde(rename = "EffectType")]
    pub effectType: i32,

    #[serde(rename = "StringVal")]
    pub stringVal: String,

    #[serde(rename = "StringValSub")]
    pub stringValSub: String,

    #[serde(rename = "Duration")]
    pub duration: f32,

    #[serde(rename = "IntVal")]
    pub intVal: i32,
}

impl Default for ScenarioSpecialEffect {
    fn default() -> Self {
        ScenarioSpecialEffect {
            effectType: 8,
            stringVal: "Morning".to_string(),
            stringValSub: "".to_string(),
            duration: 0.0,
            intVal: 0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ScenarioSoundData {
    #[serde(rename = "PlayMode")]
    pub playMode: i32,

    #[serde(rename = "Bgm")]
    pub bgm: String,

    #[serde(rename = "Se")]
    pub se: String,

    #[serde(rename = "Volume")]
    pub volume: f32,

    #[serde(rename = "SeBundleName")]
    pub seBundleName: String,

    #[serde(rename = "Duration")]
    pub duration: f32,

    #[serde(rename = "BgmBlockIndex")]
    pub bgmBlockIndex: i32,
}

impl Default for ScenarioSoundData {
    fn default() -> Self {
        ScenarioSoundData {
            playMode: 0,
            bgm: "".to_string(),
            se: "se_walk_women_001_1".to_string(),
            volume: 1.0,
            seBundleName: "".to_string(),
            duration: 0.0,
            bgmBlockIndex: 0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ScenarioSnippetLayoutMode {
    #[serde(rename = "CharacterLayoutMode")]
    pub characterLayoutMode: i32,
}

impl Default for ScenarioSnippetLayoutMode {
    fn default() -> Self {
        ScenarioSnippetLayoutMode {
            characterLayoutMode: 3,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CustomStory {
    pub file_name: String,
    pub modpack_name: String,
    pub banner_image: Option<String>,
    pub story_background: Option<String>,
    pub title_background: Option<String>,
    pub logo: Option<String>,
    pub data: Vec<CustomStoryScene>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CustomStoryScene {
    pub index: i64,
    pub data: SekaiStoriesScene,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SekaiStoriesScene {
    pub last_modified: String,
    pub background: String,
    pub text: SekaiStoriesSceneText,
    pub models: Vec<SekaiStoriesSceneModels>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SekaiStoriesSceneText {
    pub name_tag: String,
    pub dialogue: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SekaiStoriesSceneModels {
    pub from: String,
    pub character: String,
    pub model_name: String,
    pub model_transform: SekaiStoriesSceneTransform,
    pub model_expression: i32,
    pub model_pose: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SekaiStoriesSceneTransform {
    pub x: i32,
    pub y: i32,
    pub scale: f32,
}

#[derive(Debug)] // Used to create a HashMap of characters and their associated data collected from the models field of SEKAI-Stories JSON.
pub struct CharacterData {
    pub id: i32,
    pub motion_name: String,
    pub facial_name: String,
    pub costume_type: String,
}

/// Saves the AssetBundle typetree inside a modpack into output_path if set, otherwise mods/{mod_name}.ab
/// Optionally encrypts before saving. (Encryption is required for the game to read it properly.)
pub fn create_assetbundle(
    modpack: ModData,
    output_path: Option<PathBuf>,
    encrypt_ab: bool,
) -> PyResult<()> {
    let mod_name = modpack.mod_name;

    let mod_ab_path = &format!("mods/{mod_name}.ab");
    let mod_ab_path = output_path.unwrap_or(Path::new(mod_ab_path).to_path_buf());

    copy("assets/story/scenario/scenario", mod_ab_path.clone())?; // TODO: Don't hardcode

    match modpack.mod_type {
        ModType::Story(scenario_self) => Python::attach(|py| {
            let filename = CString::new("story_to_assetbundle.py").unwrap();
            let modname = CString::new("story_to_assetbundle").unwrap();

            let module =
                PyModule::from_code(py, &CString::new(PY_CODE).unwrap(), &filename, &modname)?;

            module
                .getattr("set_asset_path")?
                .call1((&mod_ab_path.to_str(),))?;

            module
                .getattr("set_target_path_id")?
                .call1((SCENARIO_PATH_ID,))?;

            module.getattr("return_typetree")?.call0()?; // Will set target_object to the correct object as a global in Python so that we can then use save_typetree

            module
                .getattr("save_typetree")?
                .call1((pythonize(py, &scenario_self)?,))?;

            if encrypt_ab {
                info!("Encrypting new AssetBundle {}", mod_ab_path.display());
                match encrypt(&mod_ab_path, &mod_ab_path) {
                    Ok(_) => {
                        info!("Encrypted AssetBundle")
                    }
                    Err(e) => {
                        error!("Could not encrypt {}: {}", mod_ab_path.display(), e)
                    }
                };
            }

            info!("Saved new AssetBundle to: {}", mod_ab_path.display());
            Ok(())
        }),
    }
}

/// Loads the AssetBundle typetree from assets/story/scenario/scenario template
pub fn load_scenario_typetree(path_id: i64) -> Result<Scenario> {
    Python::attach(|py| {
        let filename = CString::new("story_to_assetbundle.py").unwrap();
        let modname = CString::new("story_to_assetbundle").unwrap();

        let module = PyModule::from_code(py, &CString::new(PY_CODE).unwrap(), &filename, &modname)?;

        module
            .getattr("set_asset_path")?
            .call1((&"assets/story/scenario/scenario",))?;

        module.getattr("set_target_path_id")?.call1((path_id,))?;

        let typetree: Scenario =
            depythonize(&module.getattr("return_typetree")?.call0()?.extract()?)?;

        Ok(typetree)
    })
}
