mod mods;
mod routes;
mod scenario;
mod utils;

use std::{fs::File, net::SocketAddr, path::Path, sync::Arc};

use axum::{
    Router,
    body::Body,
    routing::{get, post},
};
use colored::Colorize;
use gumdrop::Options;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use log::{info, warn};
use notify_rust::Notification;
use sekai_injector::{Manager, ServerStatistics, load_injection_map, serve};
use std::io::Read;
use tokio::{sync::RwLock, task};
use tower_http::services::{ServeDir, ServeFile};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Options)]
struct CommandOptions {
    #[options(help = "print help message")]
    help: bool,
    #[options(help = "be verbose")]
    verbose: bool,
    #[options(help = "specify a specific config file")]
    config: Option<String>,
}

#[tokio::main]
async fn main() {
    let opts = CommandOptions::parse_args_default_or_exit();

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("MikuMikuLoader-log.txt")
        .build("logs")
        .expect("failed to initialize rolling file appender");

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(if opts.verbose {
            EnvFilter::new("debug,dbus=warn,zbus=warn,tracing=warn") // notify_rust is very verbose otherwise
        } else {
            EnvFilter::new("info,dbus=warn,zbus=warn,tracing=warn")
        })
        .with_writer(non_blocking)
        .init();

    info!("{}", "ハローセカイ!".green());

    let config_path = match opts.config {
        Some(config) => config,
        None => "MikuMiku.toml".to_string(),
    };

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

    let mut sekai_injector_enabled = true;

    for path in [
        &injector_config_holder.server_cert,
        &injector_config_holder.server_key,
    ] {
        if !Path::new(path).exists() {
            let message = format!(
                "{path} does not exist! Sekai-injector won't start yet. Once you have generated the certificates, restart the program."
            );

            warn!("{message}");
            Notification::new()
                .summary("MikuMikuLoader")
                .body(&message)
                .show()
                .unwrap();

            sekai_injector_enabled = false;
        }
    }

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

    let static_routes = Router::new()
        .route_service("/", ServeFile::new("static/index.html"))
        .route_service(
            "/server-status",
            ServeFile::new("static/server-status.html"),
        )
        .route_service("/custom-story", ServeFile::new("static/custom-story.html"))
        .route_service("/cert-gen", ServeFile::new("static/certificate-gen.html"))
        .fallback_service(ServeDir::new("static"));

    let api_routes = Router::new()
        .route("/total-passthrough", get(routes::total_passthrough))
        .route("/total-proxied", get(routes::total_proxied))
        .route("/total-requests", get(routes::requests))
        .route("/set-param/{:param}", get(routes::set_serve_param))
        .route("/generate-ca", post(routes::gen_ca))
        .route("/generate-cert", post(routes::gen_cert))
        .route(
            "/export-custom-story",
            post(routes::export_story_to_modpack),
        )
        .route("/local-ip", get(routes::return_local_ip))
        .route("/version", get(routes::return_version))
        .with_state(Arc::clone(&manager));

    let webui_app = static_routes.merge(api_routes);

    let webui_addr = SocketAddr::from(([0, 0, 0, 0], 3939));
    let webui_server = axum_server::bind(webui_addr).serve(webui_app.into_make_service());

    info!("MikuMikuLoader running at http://0.0.0.0:3939");

    if sekai_injector_enabled {
        let backend_task = task::spawn(sekai_injector_serve(Arc::clone(&manager)));
        tokio::join!(backend_task, webui_server).0.unwrap();
    } else {
        tokio::join!(webui_server).0.unwrap();
    }
}

async fn sekai_injector_serve(manager: Arc<RwLock<Manager>>) {
    info!("Starting sekai-injector server!");

    serve(manager).await;
}
