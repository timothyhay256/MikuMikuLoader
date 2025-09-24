use std::{
    collections::HashMap,
    env,
    fs::{self, File, create_dir},
    io::Cursor,
    path::{Path as fPath, Path as FPath},
    sync::Arc,
};

use anyhow::Result;
use axum::{
    Json,
    extract::{Path, State},
    http::Uri,
    response::IntoResponse,
};
use base64::prelude::*;
use image::{
    DynamicImage, ImageEncoder,
    codecs::png::{CompressionType, FilterType, PngEncoder},
    load_from_memory,
};
use local_ip_address::local_ip;
use log::{debug, error, info, warn};
use sekai_injector::{
    CertificateGenParams, Manager, RequestParams, generate_ca, new_self_signed_cert,
};
use serde::Deserialize;
use tokio::sync::RwLock;
use walkdir::WalkDir;

use crate::{
    StaticFile,
    assetbundle::{generate_logo, generate_screen_image, reload_assetbundle_info},
    encrypt,
    mods::{CacheInvalidDuration, InvalidateCacheEntry, ModData, ModType, reload_injections},
    notify_mml,
    scenario::{
        CharacterData, CustomStory, SCENARIO_PATH_ID, ScenarioCharacterLayout, ScenarioSnippet,
        ScenarioSpecialEffect, ScenarioTalkData, TalkCharacter, TalkMotion, create_assetbundle,
        load_scenario_typetree,
    },
    utils::{self, Model3Root},
};

#[derive(Debug, Deserialize)]
pub struct CertGenOptions {
    pub hostname: String,
    pub ip: String,
    pub cert_lifetime: i64,
    pub ca_name_input: String,
    pub ca_key_input: String,
    pub cert_name: String,
    pub cert_key_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CAGenOptions {
    pub ca_name: String,
    pub ca_lifetime: i64,
    pub ca_file_name: String,
    pub ca_key_name: String,
}

#[allow(dead_code)] // Not all items will be used, but all are required for deserialization
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Character2DS {
    id: i32,
    character_type: String,
    is_next_grade: bool,
    character_id: i32,
    unit: String,
    is_enabled_flip_display: bool,
    asset_name: Option<String>,
}

// TODO: Use SSE and channels
pub async fn total_passthrough(State(state): State<Arc<RwLock<Manager>>>) -> impl IntoResponse {
    state.read().await.statistics.request_count.0.to_string()
}

pub async fn total_proxied(State(state): State<Arc<RwLock<Manager>>>) -> impl IntoResponse {
    state.read().await.statistics.request_count.1.to_string()
}

pub async fn requests(State(state): State<Arc<RwLock<Manager>>>) -> Json<Vec<RequestParams>> {
    Json(state.read().await.statistics.requests.clone()) // TODO: This is expensive. and silly (in a bad way). dont do it.
}

pub async fn set_serve_param(
    State(state): State<Arc<RwLock<Manager>>>,
    Path(param): Path<String>,
) -> String {
    match param.as_str() {
        "start" => {
            info!("Server start requested by web");
            state.write().await.config.inject_resources = false
        }
        "stop" => {
            info!("Server stop requested by web");
            state.write().await.config.inject_resources = true
        }
        "restart" => {
            info!("Server restart requested by web");
            // TODO
        }
        _ => return "invalid command".to_string(),
    }

    "Success".to_string()
}

pub async fn toggle_mod(Path(param): Path<String>) -> impl IntoResponse {
    // Reads current mod status and just flips it. Server restart re-reads all mods in folder, so this should be fine.
    let mod_path = fPath::new(&param);

    info!("Toggle {} requested by web", mod_path.display());

    if mod_path.exists() {
        let mod_data = fs::read_to_string(mod_path).unwrap_or_else(|_| {
            panic!(
                "Could not read {}! Please try redownloading mods and fixing permissions.",
                mod_path.display()
            )
        });

        let mut mod_data: ModData = toml::from_str(&mod_data).unwrap_or_else(|_| {
            panic!(
                "{} is not formatted properly! Check if MikuMikuLoader is out of date.",
                mod_path.display()
            )
        });

        mod_data.enabled = !mod_data.enabled;

        match toml::to_string_pretty(&mod_data) {
            Ok(toml) => match std::fs::write(mod_path, toml) {
                Ok(_) => {
                    info!(
                        "Succesfully toggled {}",
                        mod_path.canonicalize().unwrap().display()
                    );
                    format!("Toggled {}", mod_path.canonicalize().unwrap().display())
                }
                Err(e) => format!("Failed to write to {:?}: {e}", mod_path.canonicalize()),
            },
            Err(e) => format!("Failed to serialize modpack into TOML: {e}"),
        }
    } else {
        format!("Couldn't find {} to toggle!", mod_path.display())
    }
}

pub async fn mod_list() -> impl IntoResponse {
    debug!("mod list requested by web");
    let mod_path = fPath::new("mods");

    let mut list: Vec<(String, String, String, bool)> = Vec::new(); // Path, name, type, enabled

    if !mod_path.exists() {
        Json(list)
    } else {
        for entry in WalkDir::new(mod_path) {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_file()
                        && entry.path().extension().and_then(|e| e.to_str()) == Some("toml")
                    {
                        let entry_data = fs::read_to_string(entry.path()).unwrap_or_else(|_| {
                            panic!(
                                "Could not read {}! Please try redownloading mods and fixing permissions.",
                                entry.path().display()
                            )
                        });

                        let mod_data: ModData = toml::from_str(&entry_data).unwrap_or_else(|_| {
                            panic!(
                                "{} is not formatted properly! Check if MikuMikuLoader is out of date.",
                                entry.path().display()
                            )
                        });

                        list.push((
                            entry.path().display().to_string(),
                            mod_data.mod_name,
                            mod_data.mod_type.variant_name().to_owned(),
                            mod_data.enabled,
                        ));
                    }
                }
                Err(ref e) => {
                    error!("Could not read {entry:?}, skipping scan. Err: {e}");
                }
            }
        }
        debug!("Returning modlist: {list:?}");
        Json(list)
    }
}

pub async fn return_local_ip() -> impl IntoResponse {
    local_ip().unwrap().to_string()
}

pub async fn return_version() -> impl IntoResponse {
    env!("CARGO_PKG_VERSION").to_string()
}

pub async fn gen_cert(Json(payload): Json<CertGenOptions>) -> impl IntoResponse {
    debug!("Received cert generation request at gen_cert endpoint: {payload:?}");

    for path in [&payload.ca_name_input, &payload.ca_key_input].iter() {
        if !fPath::new(*path).exists() {
            return Json(format!(
                "{path} does not exist! You need to generate the CA first."
            ));
        }
    }

    for path in [&payload.cert_name, &payload.cert_key_name].iter() {
        if fPath::new(*path).exists() {
            return Json(format!(
                "{path} already exists! To overwrite it, please delete it first."
            ));
        }
    }

    match new_self_signed_cert(CertificateGenParams {
        ca_cert_pem_path: &payload.ca_name_input,
        ca_key_pem_path: &payload.ca_key_input,
        target_hostname: &payload.hostname,
        target_ip: &payload.ip,
        distinguished_common_name: &payload.hostname,
        cert_file_out_path: &payload.cert_name,
        cert_key_out_path: &payload.cert_key_name,
        cert_lifetime_days: payload.cert_lifetime,
    }) {
        Ok(_) => {}
        Err(e) => return Json(format!("Failed to generate certificate: {e}")),
    }

    info!(
        "Succesfully generated certificate at {} and {}",
        &payload.cert_name, &payload.cert_key_name
    );
    Json(format!(
        "Certificate succesfully generated! It was placed in {}",
        env::current_dir().unwrap().display()
    ))
}

pub async fn gen_ca(Json(payload): Json<CAGenOptions>) -> impl IntoResponse {
    debug!("Received CA generation request at gen_ca endpoint: {payload:?}");

    for path in [&payload.ca_file_name, &payload.ca_key_name].iter() {
        if fPath::new(*path).exists() {
            return Json(format!(
                "{path} already exists! To overwrite it, please delete it first. Please note that if you do so, the program will break until you regenerate the certificates and reinstall the new CA on your device."
            ));
        }
    }

    match generate_ca(
        &payload.ca_name,
        payload.ca_lifetime,
        &payload.ca_file_name,
        &payload.ca_key_name,
    ) {
        Ok(_) => {}
        Err(e) => return Json(format!("Failed to generate CA: {e}")),
    }

    info!(
        "Succesfully generated certificate at {} and {}",
        &payload.ca_file_name, &payload.ca_key_name
    );
    Json(format!(
        "CA succesfully generated! It was placed in {}",
        env::current_dir().unwrap().display()
    ))
}

pub async fn export_story_to_modpack(
    config: utils::Config,
    asset_version: String,
    Json(payload): Json<CustomStory>,
) -> impl IntoResponse {
    // TODO: Clean up, make more efficient, make this an impl
    // TODO: Apply model transform
    // TODO: Multi character support
    info!("Exporting story to modpack and generating AssetBundles");

    let mod_name = payload.modpack_name;
    let mod_ab_path = &format!("mods/{mod_name}.ab");

    // Load character2ds
    let character2ds_file = fs::File::open("assets/character2ds.json").expect("Could not read assets/character2ds.json! Please remove the assets folder and try again to redownload assets.");

    let character2ds: Vec<Character2DS> = serde_json::from_reader(character2ds_file).expect(
        "character2ds.json is not formatted properly! Check if MikuMikuLoader is out of date.",
    );

    let character2ds_map: HashMap<String, Character2DS> = character2ds
        .into_iter()
        .filter_map(|c| c.asset_name.clone().map(|name| (name, c)))
        .collect();

    let mut injected_assets = HashMap::new();
    injected_assets.insert(
        "event_story/event_whip_2024/scenario".to_string(),
        mod_ab_path.clone(),
    );

    // Loads the template typetree which we will then modify
    let scenario_typetree = match load_scenario_typetree(SCENARIO_PATH_ID) {
        Ok(scenario_typetree) => scenario_typetree,
        Err(e) => return format!("Failed to load typetree. Is UnityPy installed? Err: {e}"),
    };

    let mut modpack = ModData {
        mod_name: mod_name.clone(),
        enabled: true,
        mod_type: crate::mods::ModType::Story(scenario_typetree),
        // TODO: Change resource_path and duration
        invalidated_assets: vec![InvalidateCacheEntry {
            resource_path: "event_story/event_whip_2024/scenario".to_string(),
            duration: CacheInvalidDuration::PermanentlyInvalid,
        }],
        injected_assets,
    };

    // Store all characters and their expressions while looping through models to be used later
    let mut character_expressions: Option<HashMap<String, CharacterData>> = None;

    let ModType::Story(adapter) = &mut modpack.mod_type;

    // Push the first background
    let bkg_name = fPath::new(&payload.data[0].data.background.clone().to_string())
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_owned();

    debug!("Pushing first background");
    adapter.firstBackground = bkg_name.clone();

    adapter
        .needBundleNames
        .push(format!("scenario/background/{bkg_name}"));

    // Push special effect data with story name and (?) MikuMikuLoader
    adapter.specialEffectData.push(ScenarioSpecialEffect {
        effectType: 8,
        stringVal: "Created with MikuMikuLoader".to_owned(),
        stringValSub: "".to_owned(),
        duration: 0.0,
        intVal: 0,
    });

    adapter.specialEffectData.push(ScenarioSpecialEffect {
        effectType: 8,
        stringVal: mod_name.to_owned(),
        stringValSub: "".to_owned(),
        duration: 0.0,
        intVal: 0,
    });

    adapter.specialEffectData.push(ScenarioSpecialEffect {
        effectType: 4,
        stringVal: "".to_owned(),
        stringValSub: "".to_owned(),
        duration: 1.0,
        intVal: 0,
    });

    // Push necessary ScenarioSnippet for above special effects

    // "MikuMikuLoader" special effect
    adapter.snippets.push(ScenarioSnippet {
        index: 0,
        action: 6,
        progressBehavior: 1,
        referenceIndex: 0,
        delay: 0.0,
    });

    // Mod name special effect
    adapter.snippets.push(ScenarioSnippet {
        index: 1,
        action: 6,
        progressBehavior: 1,
        referenceIndex: 1,
        delay: 0.0,
    });

    // Clear special effects
    adapter.snippets.push(ScenarioSnippet {
        index: 2,
        action: 7,
        progressBehavior: 1,
        referenceIndex: 0,
        delay: 0.0,
    });

    // Have the character appear TODO: Multiple character support
    adapter.snippets.push(ScenarioSnippet {
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
                let asset_prefix = {
                    if let Some(pos) = model.model_name.rfind('_') {
                        if model.model_name[..pos].contains('_') {
                            &model.model_name[..pos]
                        } else {
                            // Just one underscore indicates it doesn't have anything to trim
                            &model.model_name
                        }
                    } else {
                        // No underscores
                        &model.model_name
                    }
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

                if !adapter.appearCharacters.contains(&character_to_push) {
                    debug!("Pushing appearCharacters: {character_to_push:?}");
                    adapter // Push character to appear_characters
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
                        adapter.talkData.push(ScenarioTalkData {
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

                        adapter.layoutData.push(scenario_char_layout);

                        // If it's the last scene, we need to push an empty layoutdata. Not sure why.
                        if index == payload.data.len() - 1 {
                            debug!("Pushing final needed empty ScenarioSnippetCharacterLayout!");

                            adapter.layoutData.push(ScenarioCharacterLayout {
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
                        // match adapter.layoutData.contains(&scenario_char_layout) {
                        //     true => {}
                        //     false => adapter.layoutData.push(scenario_char_layout),
                        // };

                        // Push an ScenarioSnippet for our dialogue
                        adapter.snippets.push(ScenarioSnippet {
                            index: adapter.snippets.len() as i32,
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
                        adapter.talkData.push(ScenarioTalkData::default())
                    }
                }
            }
            None => {
                error!(
                    "No characters were correctly initialized in the last step. Tried to find {character_name} but character_expressions is None"
                );
                return "No compatible characters could be found. Please make sure all chracters in each scene are Project Sekai characters! Support for non-Sekai characters is not yet implemented.".to_string();
            }
        }
    }

    info!("Creating associated AssetBundles...");

    // Generate screen_image AssetBundle

    // Deleted when dropped, thus we need it here
    let banner_image_tmp_file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
    let banner_image = {
        match png_from_base64_str(payload.banner_image) {
            Ok(img) => {
                if let Some(image) = img {
                    let img_path = banner_image_tmp_file.path().display().to_string();
                    save_png_best_compression(&image, &img_path).unwrap();
                    Some(FPath::new(&img_path).to_path_buf())
                } else {
                    None
                }
            }
            Err(e) => {
                return format!("Banner image provided is not an valid image! Err: {e}")
                    .to_string();
            }
        }
    };

    let story_background_tmp_file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
    let story_background = {
        match png_from_base64_str(payload.story_background) {
            Ok(img) => {
                if let Some(image) = img {
                    let img_path = story_background_tmp_file.path().display().to_string();
                    save_png_best_compression(&image, &img_path).unwrap();
                    Some(FPath::new(&img_path).to_path_buf())
                } else {
                    None
                }
            }
            Err(e) => {
                return format!("Banner image provided is not an valid image! Err: {e}")
                    .to_string();
            }
        }
    };

    let title_background_tmp_file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
    let title_background = {
        match png_from_base64_str(payload.title_background) {
            Ok(img) => {
                if let Some(image) = img {
                    let img_path = title_background_tmp_file.path().display().to_string();
                    save_png_best_compression(&image, &img_path).unwrap();
                    Some(FPath::new(&img_path).to_path_buf())
                } else {
                    None
                }
            }
            Err(e) => {
                return format!("Banner image provided is not an valid image! Err: {e}")
                    .to_string();
            }
        }
    };

    // Copy template and generate new screen_image assetbundle in place of the copied original assetbundle
    let screen_image_path = format!("mods/{mod_name}-screenImage.ab");
    tokio::fs::copy("assets/story/screen_image/screen_image", &screen_image_path)
        .await
        .unwrap();

    modpack.invalidated_assets.push(InvalidateCacheEntry {
        resource_path: "event_story/event_whip_2024/screen_image".to_string(),
        duration: CacheInvalidDuration::PermanentlyInvalid,
    });

    modpack.injected_assets.insert(
        "event_story/event_whip_2024/screen_image".to_string(),
        screen_image_path.clone(),
    );

    info!("Generating screen image");
    match generate_screen_image(
        &screen_image_path,
        banner_image,
        story_background,
        title_background,
    )
    .await
    {
        Ok(_) => {
            // Encrypt new AssetBundle

            info!("Encrypting new AssetBundle {}", &screen_image_path);

            let screen_image_path = FPath::new(&screen_image_path);
            match encrypt(screen_image_path, screen_image_path) {
                Ok(_) => {
                    info!("Encrypted AssetBundle")
                }
                Err(e) => {
                    error!("Could not encrypt {}: {}", screen_image_path.display(), e)
                }
            };
        }
        Err(e) => error!("Failed to generate screen_image! Default will be used. Err: {e}"),
    };

    // Copy template and generate new logo assetbundle in place of the copied original assetbundle
    let logo_ab_path = format!("mods/{mod_name}-logo.ab");
    tokio::fs::copy("assets/event/logo/logo", &logo_ab_path)
        .await
        .unwrap();

    modpack.invalidated_assets.push(InvalidateCacheEntry {
        resource_path: "event/event_whip_2024/logo".to_string(),
        duration: CacheInvalidDuration::PermanentlyInvalid,
    });

    modpack.injected_assets.insert(
        "event/event_whip_2024/logo".to_string(),
        logo_ab_path.clone(),
    );

    // Save image to temp path since generate_logo requires an path (which is needed because UnityPy requires it)

    let logo = {
        match png_from_base64_str(payload.logo) {
            Ok(img) => img,
            Err(e) => {
                return format!("Banner image provided is not an valid image! Err: {e}")
                    .to_string();
            }
        }
    };

    let image_file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
    let img_path = image_file.path().display().to_string();
    if let Some(logo) = logo {
        save_png_best_compression(&logo, &img_path).unwrap();
        info!("Generating logo AssetBundle");
        match generate_logo(logo_ab_path.clone(), FPath::new(&img_path).to_path_buf()).await {
            Ok(_) => {}
            Err(e) => error!("Failed to generate logo! Default will be used. Err: {e}"),
        };
    }

    // Encrypt logo AssetBundle, be it the template or newly generated AssetBundle
    info!("Encrypting new AssetBundle {}", &logo_ab_path);

    let logo_ab_path = FPath::new(&logo_ab_path);
    match encrypt(logo_ab_path, logo_ab_path) {
        Ok(_) => {
            info!("Encrypted AssetBundle")
        }
        Err(e) => {
            error!("Could not encrypt {}: {}", logo_ab_path.display(), e)
        }
    };

    let file_name = {
        if payload.file_name.contains(".toml") {
            payload.file_name
        } else {
            format!("{}.toml", payload.file_name)
        }
    };

    let output_file_path = format!("mods/{file_name}");
    let output_file = fPath::new(&output_file_path);

    if !fPath::new("mods").exists() {
        match create_dir("mods") {
            Ok(_) => {}
            Err(e) => {
                error!(
                    "Could not create mods dir, will try to continue with saving but will likely fail! Err: {e}"
                )
            }
        }
    }

    if output_file.exists() {
        return format!(
            "{} already exists! Please rename your file.",
            output_file.display()
        );
    } else {
        match toml::to_string_pretty(&modpack) {
            Ok(toml) => match std::fs::write(output_file, toml) {
                Ok(_) => {
                    info!(
                        "Succesfully generated and saved modpack to {}",
                        output_file.canonicalize().unwrap().display()
                    );
                    info!(
                        "Successfully generated modpack! It was placed in {}",
                        output_file.canonicalize().unwrap().display()
                    )
                }
                Err(e) => {
                    return format!("Failed to write to {:?}: {e}", output_file.canonicalize());
                }
            },
            Err(e) => return format!("Failed to serialize modpack into TOML: {e}"),
        }
    }

    info!("Creating scenario");
    match create_assetbundle(
        modpack,
        Some(std::path::Path::new(mod_ab_path).to_path_buf()),
        true,
    ) {
        Ok(_) => {
            // Modifies injections-ab with required paths
            info!("Reloading injections!");
            match reload_injections(&config) {
                Ok(_) => {
                    info!(
                        "Reloading assetbundle info hashes! This may trigger multiple redownloads within the game."
                    );

                    match reload_assetbundle_info(&config, &asset_version).await {
                        Ok(_) => {
                            let msg = "Succesfully generated modpack and AssetBundle!".to_string();
                            info!("{msg}");
                            msg
                        }
                        Err(e) => {
                            let msg = format!("Failed to reload assetbundle info! Err: {e}");
                            error!("{msg}");
                            msg
                        }
                    }
                }
                Err(e) => {
                    let msg = format!("Failed to reload injections! Err: {e}");
                    error!("{msg}");
                    msg
                }
            }
        }
        Err(e) => format!("Failed to convert modpack to AssetBundle: {e}"),
    }
}

// Needed because UnityPy seems to take a very long time with images
fn save_png_best_compression(
    img: &DynamicImage,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;

    // Convert image to RGBA8 buffer
    let rgba8 = img.to_rgba8();
    let (width, height) = rgba8.dimensions();

    // Use best compression + adaptive filtering
    let encoder = PngEncoder::new_with_quality(
        file,
        CompressionType::Best, // max DEFLATE compression
        FilterType::Adaptive,  // usually smallest files
    );

    encoder.write_image(&rgba8, width, height, image::ExtendedColorType::Rgba8)?;
    Ok(())
}

fn png_from_base64_str(base64: Option<String>) -> Result<Option<DynamicImage>> {
    if let Some(base64) = base64 {
        let image_bytes = BASE64_STANDARD.decode(base64.as_bytes())?;
        let img = load_from_memory(&image_bytes)?;

        let mut image_buffer = Cursor::new(Vec::new());
        img.write_to(&mut image_buffer, image::ImageFormat::Png)?;
        Ok(Some(load_from_memory(&image_buffer.into_inner())?))
    } else {
        Ok(None)
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub async fn index_handler() -> impl IntoResponse {
    static_handler("/index.html".parse::<Uri>().unwrap()).await
}

pub async fn custom_story_handler() -> impl IntoResponse {
    static_handler("/custom-story.html".parse::<Uri>().unwrap()).await
}

pub async fn cert_gen_handler() -> impl IntoResponse {
    static_handler("/certificate-gen.html".parse::<Uri>().unwrap()).await
}

pub async fn server_status_handler() -> impl IntoResponse {
    static_handler("/server-status.html".parse::<Uri>().unwrap()).await
}

pub async fn mod_manager_handler() -> impl IntoResponse {
    static_handler("/mod-manager.html".parse::<Uri>().unwrap()).await
}

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/').to_string();

    StaticFile(path)
}
