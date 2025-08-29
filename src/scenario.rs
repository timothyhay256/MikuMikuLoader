use std::{
    ffi::CString,
    fs::copy,
    path::{Path, PathBuf},
};

use anyhow::Result;
use log::{error, info};
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
};

/// Contains all relevant UnityPy code for loading and exporting a typetree.
const PY_CODE: &str = include_str!("../python/story_to_assetbundle.py");

/// The path id to the scenario in the template we are using, which in this case is the whip_2024 scenario but stripped of anything but the bare min.
static SCENARIO_PATH_ID: i64 = 6343946530110770478;

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
        ModType::Story(scenario_adapter) => Python::with_gil(|py| {
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
                .call1((pythonize(py, &scenario_adapter)?,))?;

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
pub fn load_ab_typetree() -> Result<Scenario> {
    Python::with_gil(|py| {
        let filename = CString::new("story_to_assetbundle.py").unwrap();
        let modname = CString::new("story_to_assetbundle").unwrap();

        let module = PyModule::from_code(py, &CString::new(PY_CODE).unwrap(), &filename, &modname)?;

        module
            .getattr("set_asset_path")?
            .call1((&"assets/story/scenario/scenario",))?;

        module
            .getattr("set_target_path_id")?
            .call1((SCENARIO_PATH_ID,))?;

        let typetree: Scenario =
            depythonize(&module.getattr("return_typetree")?.call0()?.extract()?)?;

        Ok(typetree)
    })
}
