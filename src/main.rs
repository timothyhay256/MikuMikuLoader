mod routes;
mod scenario;
mod utils;

use std::{fs::File, net::SocketAddr, path::Path, sync::Arc};

use axum::{Router, body::Body, routing::get};
use colored::Colorize;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use log::info;
use sekai_injector::{Manager, ServerStatistics, load_injection_map, serve};
use std::io::Read;
use tokio::{sync::RwLock, task};
use tower_http::services::{ServeDir, ServeFile};

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

    // We build the manager here so that it can be reused in other routes.

    let injection_hashmap = load_injection_map(&injector_config_holder.resource_config);

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
        config: injector_config_holder,
        client,
        statistics: ServerStatistics {
            request_count: (0, 0),
            requests: Vec::new(),
        },
    }));

    let backend_task = task::spawn(sekai_injector_serve(Arc::clone(&manager)));

    let webui_app = Router::new()
        .route_service("/", ServeFile::new("static/index.html"))
        .route_service(
            "/server-status",
            ServeFile::new("static/server-status.html"),
        )
        .route("/total-passthrough", get(routes::total_passthrough))
        .with_state(Arc::clone(&manager))
        .route("/total-proxied", get(routes::total_proxied))
        .with_state(Arc::clone(&manager))
        .route("/total-requests", get(routes::requests))
        .with_state(Arc::clone(&manager))
        .route("/set-param/{:param}", get(routes::set_serve_param))
        .with_state(manager)
        .fallback_service(ServeDir::new("static"));

    let webui_addr = SocketAddr::from(([0, 0, 0, 0], 3939));
    let webui_server = axum_server::bind(webui_addr).serve(webui_app.into_make_service());

    tokio::join!(backend_task, webui_server).0.unwrap();
}

async fn sekai_injector_serve(manager: Arc<RwLock<Manager>>) {
    info!("Starting sekai-injector server!");

    serve(manager).await;
}
