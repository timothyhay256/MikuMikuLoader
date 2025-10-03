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
use log::{debug, error, info};
use sekai_injector::{
    CertificateGenParams, Manager, RequestParams, generate_ca, new_self_signed_cert,
};
use serde::Deserialize;
use tokio::{
    sync::RwLock,
    task::{self, spawn_blocking},
};
use walkdir::WalkDir;

use crate::{
    StaticFile,
    assetbundle::{generate_logo, generate_screen_image, reload_assetbundle_info},
    encrypt,
    mods::{CacheInvalidDuration, InvalidateCacheEntry, ModData, ModType, reload_injections},
    scenario::{CustomStory, SCENARIO_PATH_ID, create_assetbundle, load_scenario_typetree},
    utils::{self},
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

    // payload will be moved into closure, so we need to clone what we need now
    let mod_name = payload.modpack_name.clone();
    let payload_file_name = payload.file_name.clone();
    let payload_logo = payload.logo.clone();
    let payload_story_background = payload.story_background.clone();
    let payload_title_background = payload.title_background.clone();
    let payload_banner_image = payload.banner_image.clone();
    let mod_ab_path = format!("mods/{mod_name}.ab");

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

    let mut modpack = task::spawn_blocking(move || {
        let mut modpack = ModData {
            mod_name: payload.modpack_name.clone(),
            enabled: true,
            mod_type: crate::mods::ModType::Story(scenario_typetree),
            invalidated_assets: Vec::new(),
            injected_assets,
        };
        let ModType::Story(adapter) = &mut modpack.mod_type;

        adapter.generate_story_assetbundle(&payload);

        modpack
    })
    .await
    .expect("generate_story_assetbundle blocking task failed");

    info!("Creating associated AssetBundles...");

    // Generate screen_image AssetBundle

    // Deleted when dropped, thus we need it here
    let banner_image_tmp_file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
    let banner_image = spawn_blocking(move || match png_from_base64_str(&payload_banner_image) {
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
            error!("Banner image provided is not an valid image! Err: {e}");
            None
        }
    })
    .await
    .expect("banner image from base64 blocking task failed");

    let story_background_tmp_file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
    let story_background =
        spawn_blocking(
            move || match png_from_base64_str(&payload_story_background) {
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
                    error!("Banner image provided is not an valid image! Err: {e}");
                    None
                }
            },
        )
        .await
        .expect("story background image from base64 blocking task failed");

    let title_background_tmp_file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
    let title_background =
        spawn_blocking(
            move || match png_from_base64_str(&payload_title_background) {
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
                    error!("Banner image provided is not an valid image! Err: {e}");
                    None
                }
            },
        )
        .await
        .expect("title background image from base64 blocking task failed");

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

    spawn_blocking(move || {
        info!("Generating screen image");
        match generate_screen_image(
            &screen_image_path,
            banner_image,
            story_background,
            title_background,
        ) {
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
        fs::copy("assets/event/logo/logo", &logo_ab_path)
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
            match png_from_base64_str(&payload_logo) {
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
            match generate_logo(logo_ab_path.clone(), FPath::new(&img_path).to_path_buf()) {
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
            if payload_file_name.contains(".toml") {
                payload_file_name
            } else {
                format!("{}.toml", payload_file_name)
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
            Some(std::path::Path::new(&mod_ab_path).to_path_buf()),
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

                        match reload_assetbundle_info(&config, &asset_version) {
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
    })
    .await
    .expect("generate_screen_image blocking task failed")
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
    let encoder = PngEncoder::new_with_quality(file, CompressionType::Best, FilterType::Adaptive);

    encoder.write_image(&rgba8, width, height, image::ExtendedColorType::Rgba8)?;
    Ok(())
}

fn png_from_base64_str(base64: &Option<String>) -> Result<Option<DynamicImage>> {
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
