mod mods;
mod routes;
mod scenario;
mod utils;

use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, File, create_dir_all},
    io::Cursor,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    panic,
    path::Path,
    pin::Pin,
    sync::Arc,
};

use axum::{
    Router,
    body::Body,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};

use colored::Colorize;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use gumdrop::Options;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use log::{debug, error, info, warn};
use notify_rust::Notification;
use routes::static_handler;
use rust_embed::Embed;
use sekai_injector::{Config, Manager, ServerStatistics, load_injection_map, serve};
use serde::Deserialize;
use simple_dns_server::{Config as DConfig, RecordInfo, RecordType, SimpleDns};
use std::io::Read;
use std::io::Write;
use tokio::{sync::RwLock, task};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use utils::AssetConfig;

#[derive(Debug, Options)]
struct CommandOptions {
    #[options(help = "print help message")]
    help: bool,
    #[options(help = "be verbose")]
    verbose: bool,
    #[options(help = "specify a specific config file")]
    config: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Versions {
    app_hash: String,
    system_profile: String,
    app_version: String,
    multi_play_version: String,
    asset_version: String,
    app_version_status: String,
    data_version: String,
    asset_hash: String,
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

    // Needed since below is Windows only
    #[allow(unused_mut)]
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

    let injector_config_holder = update_injection_appversion(injector_config_holder); // Read versions.json and update resource_prefix if it is necessary.

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
            notify_mml(&message);

            sekai_injector_enabled = false;
        }
    }

    // Configure and create DNS server

    let dns_server = SimpleDns::try_load(configure_dns(&injector_config_holder))
        .await
        .expect("Failed to configure DNS server. Please file a bug report.");

    let dns_server_handle = task::spawn(async move {
        info!("Starting DNS server");

        match dns_server.run().await {
            Ok(_) => {
                info!("DNS server exited normally");
                Ok(())
            }
            Err(e) => {
                error!("DNS server exited with error: {e}");
                Err(e)
            }
        }
    });

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
        .route_service("/", get(routes::index_handler))
        .route_service("/server-status", get(routes::server_status_handler))
        .route_service("/custom-story", get(routes::custom_story_handler))
        .route_service("/cert-gen", get(routes::cert_gen_handler))
        .route_service("/mod-manager", get(routes::mod_manager_handler))
        .route("/{*file}", get(static_handler));

    let api_routes = Router::new()
        .route("/total-passthrough", get(routes::total_passthrough))
        .route("/total-proxied", get(routes::total_proxied))
        .route("/total-requests", get(routes::requests))
        .route("/set-param/{:param}", get(routes::set_serve_param))
        .route("/toggle-mod/{:param}", get(routes::toggle_mod))
        .route("/generate-ca", post(routes::gen_ca))
        .route("/generate-cert", post(routes::gen_cert))
        .route(
            "/export-custom-story",
            post(routes::export_story_to_modpack),
        )
        .route("/local-ip", get(routes::return_local_ip))
        .route("/version", get(routes::return_version))
        .route("/mod-list", get(routes::mod_list))
        .with_state(Arc::clone(&manager));

    let webui_app = static_routes.merge(api_routes);

    let webui_addr = SocketAddr::from(([0, 0, 0, 0], 3939));
    let webui_server = axum_server::bind(webui_addr).serve(webui_app.into_make_service());

    notify_mml("MikuMikuLoader running at http://0.0.0.0:3939");

    match update_assets(config_holder.advanced.assets).await {
        Ok(_) => {}
        Err(e) => error!("Could not update assets: {e}\n"),
    };

    #[cfg(not(debug_assertions))]
    // Don't open browser in debug mode, because that would be really annoying.
    match webbrowser::open("http://127.0.0.1:3939") {
        Ok(_) => {}
        Err(e) => {
            let message = format!(
                "Could not open default browser: {e} Please visit http://127.0.0.1:3939 to use MikuMikuLoader."
            );
            error!("{message}");
            notify_mml(&message);
        }
    }

    if sekai_injector_enabled {
        let backend_task = task::spawn(sekai_injector_serve(Arc::clone(&manager)));

        let (backend_result, dns_result, webui_result) =
            tokio::join!(backend_task, dns_server_handle, webui_server);

        match backend_result {
            Ok(_) => {}
            Err(e) => {
                notify_mml(&format!(
                    "You might need to run as root/admin.\nCould not start MikuMikuLoader: {e}\n\nCheck the log for more information!"
                ));
            }
        }

        match dns_result {
            Ok(Ok(_)) => println!("DNS server task completed successfully."),
            Ok(Err(e)) => eprintln!("DNS server error: {e}"),
            Err(e) => eprintln!("DNS server task panicked: {e}"),
        }

        webui_result.unwrap();
    } else {
        let (webui_result, dns_result) = tokio::join!(webui_server, dns_server_handle);

        webui_result.unwrap();

        match dns_result {
            Ok(Ok(_)) => println!("DNS server task completed successfully."),
            Ok(Err(e)) => eprintln!("DNS server error: {e}"),
            Err(e) => eprintln!("DNS server task panicked: {e}"),
        }
    }
}

async fn sekai_injector_serve(manager: Arc<RwLock<Manager>>) {
    info!("Starting sekai-injector server!");

    serve(manager).await;
}

pub fn notify_mml(body: &str) {
    info!("{body}");

    if let Err(e) = Notification::new()
        .appname("MikuMikuLoader")
        .summary("MikuMikuLoader")
        .body(body)
        .show()
    {
        error!("Could not show desktop notification: {e}");
    }
}

// Reads Sekai Injector config and returns an DConfig that can be used to spawn a DNS server
pub fn configure_dns(config: &Config) -> DConfig {
    let bind = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 53);

    let record_info = RecordInfo {
        name: "@".to_string(),
        records: vec![config.target_ip.to_string()],
        record_type: RecordType::A,
    };

    let mut domains = BTreeMap::new();
    domains.insert(config.upstream_host.clone(), vec![record_info]);

    let conf = DConfig { bind, domains };
    info!("DNS server config: {conf:?}");
    conf
}

pub fn update_injection_appversion(
    injector_config: sekai_injector::Config,
) -> sekai_injector::Config {
    if injector_config.resource_prefix.is_some() {
        warn!("Prefix is manually set, will not override with up-to-date apphash and version!");
        injector_config
    } else {
        let versions_path = "assets/versions.json";
        let mut injector_config = injector_config.clone();

        debug!("Trying to read version data from {versions_path}");

        let version_file = fs::File::open(versions_path).unwrap_or_else(|_| panic!("Could not read {}! Please remove the assets folder and try again to redownload assets.", &versions_path));

        let versions: Versions = serde_json::from_reader(version_file).unwrap_or_else(|_| {
            panic!(
                "{versions_path} is not formatted properly! Check if MikuMikuLoader is out of date."
            )
        });

        let prefix = format!("/{}/{}", versions.app_version, versions.app_hash);
        debug!("Setting prefix to {prefix}");

        injector_config.resource_prefix = Some(prefix);
        injector_config
    }
}

pub async fn update_assets(asset_config: AssetConfig) -> Result<(), Box<dyn Error>> {
    // TODO: Implement retries
    notify_mml("Checking assets for updates...");

    let client = reqwest::Client::new();
    let mut tasks = FuturesUnordered::<Pin<Box<dyn Future<Output = String>>>>::new();

    for (list, base_url) in [
        asset_config.needed_asset_files,
        asset_config.needed_template_files,
        asset_config.needed_live2d_files,
    ]
    .iter()
    .zip([
        asset_config.common_asset_url,
        asset_config.template_asset_url,
        asset_config.live2d_asset_url,
    ]) {
        let list = list.clone();

        for asset in list {
            let client = client.clone();
            let url = format!("https://{}/{asset}", &base_url.clone());

            tasks.push(Box::pin(async move {
                let mut skip_download = false;

            info!("Updating {url}");

            let resp_result = client.head(&url).send().await;
            let resp = match resp_result {
                Ok(resp) => { resp }
                Err(e) => panic!("Request failed: {e:?}"),
            };

            let mut new_etag_val: Option<&str> = None;

            if let Some(etag) = resp.headers().get("ETag") {
                debug!("{} ETag: {}", asset, etag.to_str().unwrap());
                new_etag_val = Some(etag.to_str().unwrap());
            } else {
                warn!("Upstream server doesn't support etag, redownloading assets!");
            }

            let existing_etag_path = format!("assets/{asset}.etag");
            let existing_etag_path = Path::new(&existing_etag_path);
            let mut etag_needs_update = true;

            if existing_etag_path.exists() {
                let existing_etag_val = {
                    let mut contents = String::new();

                    File::open(existing_etag_path)
                        .expect("Cannot read {}, despite knowing it exists.")
                        .read_to_string(&mut contents)
                        .unwrap();

                    contents
                };

                if existing_etag_val == new_etag_val.unwrap_or("") {
                    // If it's a None, then just make sure this will eval false.
                    info!("{} is still up to date.", existing_etag_path.display());
                    skip_download = true;
                    etag_needs_update = false;
                }
            }

            if !skip_download {
                info!("Downloading {asset}");
                let resp = client.get(url).send().await.unwrap();

                // If the asset has parent directory(s), create them
                let asset_dir = match asset.rfind('/') {
                    Some(idx) => &asset[..idx],
                    None => "",
                };

                match create_dir_all(format!("assets/{asset_dir}")) {
                    Ok(_) => {}
                    Err(e) => {
                        error!(
                            "Cannot create assets/{asset_dir}: {e}. Trying to download anyway..."
                        )
                    }
                }

                let mut file = std::fs::File::create(format!("assets/{asset}")).expect("Failed to write asset to file");
                let mut content = Cursor::new(resp.bytes().await.unwrap());
                std::io::copy(&mut content, &mut file).expect("Failed to write asset to file");
            }

            if etag_needs_update && new_etag_val.is_some() {
                // Update etag
                debug!("Storing etag for {asset}");

                match File::create(format!("assets/{asset}.etag")) {
                    Ok(mut etag_file) => match write!(etag_file, "{}", new_etag_val.unwrap()) {
                        Ok(_) => {}
                        Err(e) => {
                            error!(
                                "Failed to write assets/{asset}.etag! Asset will always be redownloaded. Err: {e}"
                            )
                        }
                    },
                    Err(e) => error!(
                        "Failed to write assets/{asset}.etag! Asset will always be redownloaded. Err: {e}"
                    ),
                }
            }

            format!("Updated {asset}")
            }));
        }
    }

    // Wait for all the downloads to complete
    while let Some(result) = tasks.next().await {
        info!("{result}");
    }

    info!("Finished updating assets!");

    Ok(())
}

fn decrypt(infile: &Path, outfile: &Path) -> std::io::Result<()> {
    // All credit for figuring out decryption goes to https://github.com/mos9527

    let mut fin = File::open(infile)?;
    let mut magic = [0u8; 4];
    fin.read_exact(&mut magic)?;

    let mut fout = File::create(outfile)?;

    if magic == [0x10, 0x00, 0x00, 0x00] {
        for _ in (0..128).step_by(8) {
            let mut block = [0u8; 8];
            fin.read_exact(&mut block)?;
            (0..5).for_each(|i| {
                block[i] = !block[i];
            });
            fout.write_all(&block)?;
        }
        let mut buffer = [0u8; 8];
        while fin.read_exact(&mut buffer).is_ok() {
            fout.write_all(&buffer)?;
        }
    } else {
        println!("copy {infile:?} -> {outfile:?}");
        let mut buffer = [0u8; 8];
        fout.write_all(&magic)?; // Write already-read magic
        while fin.read_exact(&mut buffer).is_ok() {
            fout.write_all(&buffer)?;
        }
    }

    Ok(())
}

fn encrypt(infile: &Path, outfile: &Path) -> std::io::Result<()> {
    let mut fin = File::open(infile)?;
    let mut fout = File::create(outfile)?;

    // Write the magic number
    fout.write_all(&[0x10, 0x00, 0x00, 0x00])?;

    // Encrypt the first 128 bytes (16 blocks of 8 bytes)
    for _ in (0..128).step_by(8) {
        let mut block = [0u8; 8];
        if fin.read_exact(&mut block).is_ok() {
            (0..5).for_each(|i| {
                block[i] = !block[i];
            });
            fout.write_all(&block)?;
        } else {
            break; // File is smaller than 128 bytes
        }
    }

    // Copy the rest of the file as-is
    let mut buffer = [0u8; 8];
    while fin.read_exact(&mut buffer).is_ok() {
        fout.write_all(&buffer)?;
    }

    Ok(())
}

#[derive(Embed)]
#[folder = "static/"]
struct Asset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Asset::get(path.as_str()) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        }
    }
}
