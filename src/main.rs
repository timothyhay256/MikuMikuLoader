mod utils;
use std::{
    fs::{self, File},
    net::SocketAddr,
    path::Path,
    sync::Arc,
};

use axum::{Router, body::Body, routing::get};
use colored::Colorize;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use log::info;
use sekai_injector::{Manager, load_injection_map, serve};
use std::io::Read;
use tokio::{sync::RwLock, task};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("{}", "ハローセカイ!".green());

    let config_path = Path::new("MikuMiku.toml");

    let mut config_file_contents = String::new();
    let mut config_file =
        File::open(config_path).expect("Could not open MikuMiku.toml. Do I have permission?");
    config_file
        .read_to_string(&mut config_file_contents)
        .expect(
            "The config file contains non UTF-8 characters, what in the world did you put in it??",
        );
    let config_holder: utils::Config = toml::from_str(&config_file_contents)
        .expect("The config file was not formatted properly and could not be read.");

    let injector_config_path = Path::new(&config_holder.advanced.sekai_injector_config_path);

    let mut injector_file_contents = String::new();
    let mut injector_config_file = File::open(injector_config_path)
        .expect("Could not open config file. Do I have permission?");
    injector_config_file
        .read_to_string(&mut injector_file_contents)
        .expect(
            "The config file contains non UTF-8 characters, what in the world did you put in it??",
        );
    let injector_config_holder: sekai_injector::Config = toml::from_str(&injector_file_contents)
        .expect("The config file was not formatted properly and could not be read.");

    let backend_task = task::spawn(sekai_injector_serve(injector_config_holder));

    let webui_app = Router::new()
        .route(
            "/",
            get(|| async { fs::read_to_string("static/index.html").unwrap() }),
        )
        .route(
            "/server-status",
            get(|| async { fs::read_to_string("static/server-status.html").unwrap() }),
        )
        .fallback_service(ServeDir::new("static"));

    let webui_addr = SocketAddr::from(([0, 0, 0, 0], 3939));
    let webui_server = axum_server::bind(webui_addr).serve(webui_app.into_make_service());

    tokio::join!(backend_task, webui_server);
}

async fn sekai_injector_serve(config: sekai_injector::Config) {
    info!("Starting sekai-injector server!");
    // We build the manager here so that it can be reused in other routes.

    let injection_hashmap = load_injection_map(&config.resource_config);

    // Create a client to handle making HTTPS requests
    let https = HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_or_http()
        .enable_all_versions()
        .build();

    let client = Client::builder(TokioExecutor::new()).build::<_, Body>(https);

    // Create manager containing our config and injection_hashmap and HTTPS client
    let manager = Arc::new(RwLock::new(Manager {
        injection_hashmap,
        config,
        client,
        statistics: (0, 0),
    }));

    serve(manager).await;
}
