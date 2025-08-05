use std::{
    collections::HashMap,
    env,
    fs::{self, create_dir},
    io::Read,
    path::Path as fPath,
    sync::Arc,
};

use axum::{
    Json,
    extract::{Path, State},
    http::Uri,
    response::IntoResponse,
};
use local_ip_address::local_ip;
use log::{debug, error, info};
use sekai_injector::{
    CertificateGenParams, Manager, RequestParams, generate_ca, new_self_signed_cert,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use walkdir::WalkDir;

use crate::{
    StaticFile,
    mods::{CacheInvalidDuration, InvalidateCacheEntry, ModData, ModType, create_assetbundle},
    notify_mml,
    scenario::{
        CharacterData, CustomStory, ScenarioAdapter, ScenarioAdapterCharacterLayout,
        ScenarioAdapterTalkData, ScenarioAdapterTalkDataMotion,
    },
    utils::{self, BuildMotionData},
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

#[derive(Debug, Serialize)]
pub struct ModList {
    name: String,
    enabled: bool,
    mod_type: ModType,
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

pub async fn export_story_to_modpack(Json(payload): Json<CustomStory>) -> impl IntoResponse {
    // TODO: Apply model transform
    info!("Received story export to modpack request at export_story endpoint: {payload:?}");

    // Load character2ds
    let character2ds_file = fs::File::open("assets/character2ds.json").expect("Could not read assets/character2ds.json! Please remove the assets folder and try again to redownload assets.");

    let character2ds: Vec<Character2DS> = serde_json::from_reader(character2ds_file).expect(
        "character2ds.json is not formatted properly! Check if MikuMikuLoader is out of date.",
    );

    let character2ds_map: HashMap<String, Character2DS> = character2ds
        .into_iter()
        .filter_map(|c| c.asset_name.clone().map(|name| (name, c)))
        .collect();

    let mut modpack = ModData {
        mod_name: payload.file_name.replace(".toml", ""),
        enabled: true,
        mod_type: crate::mods::ModType::Story(ScenarioAdapter::default()),
        // TODO: Change resource_path and duration
        invalidated_assets: vec![InvalidateCacheEntry {
            resource_path: "/android/event_story/event_whip_2024/scenario".to_string(),
            duration: CacheInvalidDuration::PermanentlyInvalid,
        }],
    };

    // Store all characters and their expressions while looping through models to be used later
    let mut character_expressions: Option<HashMap<String, CharacterData>> = None;

    // We don't use an implementation because the CustomStory type is likely to change often, so we just do it all here
    let ModType::Story(adapter) = &mut modpack.mod_type;

    // Push the first background
    adapter.first_background = fPath::new(&payload.data[0].data.background.clone().to_string())
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_owned();

    // Loop through all the scenes to push the relevant data
    for scene in payload.data.iter() {
        // Populate appear_characters, and use the grabbed id to populate talk_data at the same time
        for model in &scene.data.models {
            if model.from == "sekai" {
                // Keep trying to remove costume name, looping in case there are multiple subsets to the costume until we find a match.
                // TODO: Why in the world does character2ds include multiple identical entries to characters? What are the differences?
                let mut asset_prefix = model.model_name.as_str();
                let character_id = loop {
                    if let Some(item) = character2ds_map.get(asset_prefix) {
                        break item.character_id;
                    }

                    match asset_prefix.rfind('_') {
                        Some(idx) => asset_prefix = &asset_prefix[..idx],
                        None => {
                            error!(
                                "Could not find a matching Character2DId for {}, using Miku! character2ds_map: {character2ds_map:?}",
                                model.model_name
                            );
                            break 21;
                        }
                    }
                };

                // Use BuildMotionData to determine the expression and pose, since SEKAI-Stories uses an index in the serialized JSON
                character_expressions.get_or_insert_with(HashMap::new).insert(model.character.clone(), {
                    let char_map = utils::build_character_map();

                    if let Some(full_id) = char_map.get(&model.character) {
                        let build_motion_data_path =
                            format!("assets/{full_id}/{}_motion_base/BuildMotionData.json", full_id.replace("_", ""));

                        debug!("Trying to read build motion data from {build_motion_data_path}");

                        let character_build_motion_file = fs::File::open(&build_motion_data_path).unwrap_or_else(|_| panic!("Could not read {}! Please remove the assets folder and try again to redownload assets.", &build_motion_data_path));

                        let character_build_motion: BuildMotionData = serde_json::from_reader(
                            character_build_motion_file,
                        )
                        .unwrap_or_else(|_| panic!("{build_motion_data_path} is not formatted properly! Check if MikuMikuLoader is out of date."));

                        // SEKAI-Stories starts at 0
                        let mut result_expression = None;
                        let mut result_pose = None;

                        for (pos, expression) in
                            character_build_motion.expressions.iter().enumerate()
                        {
                            if pos == model.model_expression as usize {
                                result_expression = Some(expression.clone())
                            }
                        }

                        for (pos, pose) in character_build_motion.motions.iter().enumerate() {
                            if pos == model.model_pose as usize {
                                result_pose = Some(pose.clone())
                            }
                        }

                        let result_expression = {
                            match result_expression {
                                Some(expression) => expression,
                                None => {
                                    if model.model_expression == 99999 { // SEKAI-Stories uses this as the default expression definition
                                        "face_normal_01".to_string()
                                    } else {
                                        error!(
                                        "Could not find expression {}! Please create a bug report. The expression will be replaced with face_cry_01",
                                        model.model_expression
                                    );
                                        "face_cry_01".to_string()
                                    }
                                }
                            }
                        };

                        let result_pose = {
                            match result_pose {
                                Some(pose) => pose,
                                None => {
                                    if model.model_pose == 99999 { // SEKAI-Stories uses this as the default pose definition
                                        "w-adult-blushed01".to_string() // TODO: Find a better default pose.
                                    } else {
                                        error!(
                                        "Could not find expression {}! Please create a bug report. The expression will be replaced with w-cute-glad01",
                                        model.model_pose
                                    );
                                        "w-cute-glad01".to_string()
                                    }
                                }
                            }
                        };

                        CharacterData {
                            id: character_id,
                            motion_name: result_pose,
                            facial_name: result_expression,
                        }
                    } else {
                        error!(
                            "Could not find character {}! Please create a bug report. The expression will be replaced with face_cry_01, and the pose will be replaced with w-cute-glad01",
                            "face_cry_01"
                        );

                        CharacterData {
                            id: 286,
                            motion_name: "w-cute-glad01".to_string(),
                            facial_name: "face_cry_01".to_string()
                        }
                    }
                });

                let character_to_push = crate::scenario::ScenarioAdapterAppearCharacters {
                    character_2d_id: { character_id },
                    character_costume: model.model_name.clone(),
                };

                if !adapter.appear_characters.contains(&character_to_push) {
                    adapter // Push character to appear_characters
                        .appear_characters
                        .push(crate::scenario::ScenarioAdapterAppearCharacters {
                            character_2d_id: { character_id },
                            character_costume: model.model_name.clone(),
                        });
                }

                // TODO: Actually apply offsets here
                adapter
                    .character_layout
                    .push(ScenarioAdapterCharacterLayout {
                        ..Default::default()
                    })
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
                        adapter.talk_data.push(ScenarioAdapterTalkData {
                            // Push character to talk_data (which will have other fields filled later)
                            character_2d_id: character.id,
                            display_name: capitalize(character_name),
                            text: scene.data.text.dialogue.clone(),
                            motion: ScenarioAdapterTalkDataMotion {
                                motion_name: character.motion_name.clone(),
                                facial_name: character.facial_name.clone(),
                            },
                            ..Default::default()
                        });
                    }
                    None => {
                        error!(
                            "Could not find {character_name} in {character_expressions:?}! Please file a bug report. Default character will be used."
                        );
                        adapter.talk_data.push(ScenarioAdapterTalkData::default())
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

    info!("Converting modpack to AssetBundle...");

    match create_assetbundle(modpack) {
        Ok(_) => "Succesfully generated modpack and AssetBundle!".to_string(),
        Err(e) => format!("Failed to convert modpack to AssetBundle: {e}"),
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
