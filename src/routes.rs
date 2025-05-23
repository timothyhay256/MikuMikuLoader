use std::{env, path::Path as fPath, sync::Arc};

use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use local_ip_address::local_ip;
use log::{debug, info};
use sekai_injector::{
    CertificateGenParams, Manager, RequestParams, generate_ca, new_self_signed_cert,
};
use serde::Deserialize;
use tokio::sync::RwLock;

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
    debug!(
        "Received cert generation request at gen_cert endpoint: {:?}",
        payload
    );

    for path in [&payload.ca_name_input, &payload.ca_key_input].iter() {
        if !fPath::new(*path).exists() {
            return Json(format!(
                "{} does not exist! You need to generate the CA first.",
                path
            ));
        }
    }

    for path in [&payload.cert_name, &payload.cert_key_name].iter() {
        if fPath::new(*path).exists() {
            return Json(format!(
                "{} already exists! To overwrite it, please delete it first.",
                path
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
    debug!(
        "Received CA generation request at gen_ca endpoint: {:?}",
        payload
    );

    for path in [&payload.ca_file_name, &payload.ca_key_name].iter() {
        if fPath::new(*path).exists() {
            return Json(format!(
                "{} already exists! To overwrite it, please delete it first. Please note that if you do so, the program will break until you regenerate the certificates and reinstall the new CA on your device.",
                path
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
