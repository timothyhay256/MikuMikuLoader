mod mods;
mod routes;
mod scenario;
mod utils;

use std::{fs::File, net::SocketAddr, panic, path::Path, sync::Arc};

use axum::{
    Router,
    body::Body,
    routing::{get, post},
};
use colored::Colorize;
use gumdrop::Options;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use log::{error, info, warn};
use notify_rust::Notification;
use sekai_injector::{Manager, ServerStatistics, load_injection_map, serve};
use std::io::Read;
use tokio::{sync::RwLock, task};
use tower_http::services::{ServeDir, ServeFile};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

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

    let mut ansi_enabled = true;

    #[cfg(windows)] // Allow Windows users to view colored output
    match ansi_term::enable_ansi_support() {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to enable ansi support for Windows cmd, disabling colors.");
            ansi_enabled = false;
        }
    }

    // Layer for stdout with ANSI colors
    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout) // or std::io::stderr if preferred
        .with_ansi(ansi_enabled)
        .with_filter(if opts.verbose {
            EnvFilter::new("debug,dbus=warn,zbus=warn,tracing=warn")
        } else {
            EnvFilter::new("info,dbus=warn,zbus=warn,tracing=warn")
        });

    // Layer for non_blocking writer without ANSI
    let non_blocking_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_filter(if opts.verbose {
            EnvFilter::new("debug,dbus=warn,zbus=warn,tracing=warn")
        } else {
            EnvFilter::new("info,dbus=warn,zbus=warn,tracing=warn")
        });

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(non_blocking_layer)
        .init();

    panic::set_hook(Box::new(|panic_info| {
        // Hook panics so those running outside a terminal window can still see errors in the logs
        if let Some(location) = panic_info.location() {
            error!(
                "Panic occurred in file '{}' at line {}: {}",
                location.file(),
                location.line(),
                panic_info
            );
        } else {
            error!("Panic occurred: {panic_info}");
        }
    }));

    info!("{}", "ハローセカイ!".green());

    let config_path = match opts.config {
        Some(config) => config,
        None => "MikuMiku.toml".to_string(),
    };

    let mut config_file_contents = String::new();

    let config_holder: utils::Config = match File::open(config_path) {
        Ok(mut file) => {
            file.read_to_string(&mut config_file_contents).expect("The config file contains non UTF-8 characters, what in the world did you put in it??");
            toml::from_str(&config_file_contents)
                .expect("The config file was not formatted properly and could not be read.")
        }
        Err(_) => {
            warn!("No MikuMikuLoader config found, using defaults!");
            utils::Config::default()
        }
    };

    let injector_config_path = Path::new(&config_holder.advanced.sekai_injector_config_path);

    let mut injector_file_contents = String::new();

    let injector_config_holder: sekai_injector::Config = match File::open(injector_config_path) {
        Ok(mut file) => {
            file.read_to_string(&mut injector_file_contents)
        .expect(
            "The config file contains non UTF-8 characters, what in the world did you put in it??",
        );
            toml::from_str(&injector_file_contents)
                .expect("The config file was not formatted properly and could not be read.")
        }
        Err(_) => {
            warn!("No sekai injector config found, using defaults!");
            sekai_injector::Config::default()
        }
    };

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

    match Notification::new()
        .summary("MikuMikuLoader")
        .body("MikuMikuLoader running at http://0.0.0.0:3939")
        .show()
    {
        Ok(_) => {}
        Err(e) => {
            error!("Could not show desktop notification: {e}")
        }
    }

    #[cfg(not(debug_assertions))]
    // Don't open browser in debug mode, because that would be really annoying.
    match webbrowser::open("http://127.0.0.1:3939") {
        Ok(_) => {}
        Err(e) => {
            let message = format!(
                "Could not open default browser: {e} Please visit http://127.0.0.1:3939 to use MikuMikuLoader."
            );
            error!("{message}");
            match Notification::new()
                .summary("MikuMikuLoader")
                .body(&message)
                .show()
            {
                Ok(_) => {}
                Err(e) => {
                    error!("Could not show desktop notification: {e}")
                }
            }
        }
    }

    if sekai_injector_enabled {
        let backend_task = task::spawn(sekai_injector_serve(Arc::clone(&manager)));
        match tokio::join!(backend_task, webui_server).0 {
            Ok(_) => {}
            Err(e) => {
                Notification::new()
                    .summary("MikuMikuLoader")
                    .body(&format!(
                        "You might need to run as root/admin.\nCould not start MikuMikuLoader: {e}\n\nCheck the log for more information!"
                    ))
                    .timeout(0)
                    .show()
                    .unwrap();
            }
        };
    } else {
        tokio::join!(webui_server).0.unwrap();
    }
}

async fn sekai_injector_serve(manager: Arc<RwLock<Manager>>) {
    info!("Starting sekai-injector server!");

    serve(manager).await;
}
