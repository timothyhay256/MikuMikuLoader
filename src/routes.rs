use std::{collections::HashMap, default, env, fs, path::Path as fPath, sync::Arc};

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
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{
    StaticFile,
    mods::{ModData, ModType},
    notify_mml,
    scenario::{CustomStory, ScenarioAdapter, ScenarioAdapterTalkData, SekaiStoriesScene},
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
        _ => return "invalid command".to_string(),
    }

    "Success".to_string()
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
        mod_type: crate::mods::ModType::Story(ScenarioAdapter::default()),
    };

    // We don't use an implementation because the CustomStory type is likely to change often, so we just do it all here
    let ModType::Story(adapter) = &mut modpack.mod_type;

    // Push the first background
    adapter.first_background = fPath::new(&payload.data[0].data.background.clone().to_string())
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_owned();

    // Loop through all the scenes to push the relevant data
    for scene in payload.data {
        // Populate appear_characters, and use the grabbed id to populate talk_data at the same time
        for model in scene.data.models {
            if model.from == "sekai" {
                // Removes costume name from character name
                let asset_prefix = match model.modelName.rfind('_') {
                    Some(idx) => &model.modelName[..idx],
                    None => &model.modelName,
                };

                let character_id = {
                    match character2ds_map.get(asset_prefix) {
                        Some(item) => item.character_id,
                        None => {
                            error!(
                                "Could not find a matching Character2DId for {}, using Miku!",
                                model.character
                            );
                            21
                        }
                    }
                };

                adapter // Push character to appear_characters
                    .appear_characters
                    .push(crate::scenario::ScenarioAdapterAppearCharacters {
                        character_2d_id: { character_id },
                        character_costume: model.modelName,
                    });

                adapter.talk_data.push(ScenarioAdapterTalkData {
                    // Push character to talk_data (which will have other fields filled later)
                    character_2d_id: character_id,
                    display_name: model.character.to_uppercase(),
                    ..Default::default()
                })
            } else {
                error!("Only characters from Project Sekai are currently supported!");
                notify_mml(
                    "Found a non-Sekai character in cust story. Only characters from Project Sekai are currently supported!",
                );
            }
        }
    }

    Json("Success")
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

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/').to_string();

    StaticFile(path)
}
