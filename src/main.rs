mod assetbundle;
mod mods;
mod routes;
mod scenario;
mod utils;

use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, File, create_dir_all},
    io::{Cursor, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    panic,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
};

use anyhow::Result;
use axum::{
    Router,
    body::Body,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use colored::Colorize;
use futures::{StreamExt, stream::FuturesUnordered};
use gumdrop::Options;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use log::{debug, error, info, warn};
use notify_rust::Notification;
use routes::static_handler;
use rust_embed::Embed;
use sekai_injector::{Config, Manager, ServerStatistics, load_injection_maps, serve};
use serde::Deserialize;
use simple_dns_server::{Config as DConfig, RecordInfo, RecordType, SimpleDns};
use tokio::{sync::RwLock, task};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use utils::Config as MMLConfig;

use crate::{
    assetbundle::{decrypt_aes_cbc, encrypt_aes_cbc, get_apimanager_keys, reload_assetbundle_info},
    mods::{ModData, create_assetbundle},
};

#[derive(Debug, Options)]
struct CommandOptions {
    #[options(help = "print help message")]
    help: bool,
    #[options(help = "be verbose")]
    verbose: bool,
    #[options(help = "specify a specific config file")]
    config: Option<String>,

    #[options(command)]
    command: Option<Command>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // All fields are needed to deserialize, but only two are actually read
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

#[derive(Debug, Options)]
enum Command {
    #[options(help = "decrypt assetbundle")]
    AbDecrypt(DecryptOptions),

    #[options(help = "encrypt assetbundle")]
    AbEncrypt(EncryptOptions),

    #[options(help = "decrypt assetbundle info")]
    AbInfoDecrypt(DecryptOptions),

    #[options(help = "encrypt assetbundle info")]
    AbInfoEncrypt(EncryptOptions),

    #[options(help = "generate assetbundle from modpack")]
    GenAssetBundle(GenAssetBundle),

    #[options(help = "reload assetbundle info cache to force asset reloading in game")]
    ReloadAbInfo(ReloadAbInfo),
}

#[derive(Debug, Options)]
struct DecryptOptions {
    #[options(help = "file to decrypt", required)]
    encrypted_path: PathBuf,

    #[options(help = "output file", required)]
    output: PathBuf,
}

#[derive(Debug, Options)]
struct EncryptOptions {
    #[options(help = "file to encrypt", required)]
    decrypted_path: PathBuf,

    #[options(help = "output file", required)]
    output: PathBuf,
}

#[derive(Debug, Options)]
struct GenAssetBundle {
    #[options(help = "path to assetbundle", required)]
    assetbundle_path: PathBuf,

    #[options(help = "output file", required)]
    output: PathBuf,
}

#[derive(Debug, Options)]
struct ReloadAbInfo {}

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

    info!("{}", "ハローセカイ!".green());

    if let Some(Command::AbDecrypt(ref decrypt_options)) = opts.command {
        info!(
            "Decrypting assetbundle {} into {}",
            decrypt_options.encrypted_path.display(),
            decrypt_options.output.display()
        );
        info!(
            "All credit for reverse engineering assetbundle encryption goes to https://github.com/mos9527"
        );

        match decrypt(&decrypt_options.encrypted_path, &decrypt_options.output) {
            Ok(_) => {
                info!("Output saved to {}", decrypt_options.output.display())
            }
            Err(e) => {
                error!(
                    "Could not decrypt {}: {}",
                    decrypt_options.encrypted_path.display(),
                    e
                )
            }
        };
        return;
    } else if let Some(Command::AbEncrypt(ref encrypt_options)) = opts.command {
        info!(
            "Encrypting assetbundle {} into {}",
            encrypt_options.decrypted_path.display(),
            encrypt_options.output.display()
        );
        info!(
            "All credit for reverse engineering assetbundle encryption goes to https://github.com/mos9527"
        );

        match encrypt(&encrypt_options.decrypted_path, &encrypt_options.output) {
            Ok(_) => {
                info!("Output saved to {}", encrypt_options.output.display())
            }
            Err(e) => {
                error!(
                    "Could not decrypt {}: {}",
                    encrypt_options.decrypted_path.display(),
                    e
                )
            }
        };
        return;
    } else if let Some(Command::AbInfoDecrypt(ref decrypt_options)) = opts.command {
        info!(
            "Encrypting assetbundle {} into {}",
            decrypt_options.encrypted_path.display(),
            decrypt_options.output.display()
        );

        let mut input_file =
            File::open(&decrypt_options.encrypted_path).expect("Could not read input file.");

        let mut output_file =
            File::create(&decrypt_options.output).expect("Could not open output file for writing.");

        let mut byte_buffer = Vec::new();
        input_file
            .read_to_end(&mut byte_buffer)
            .expect("Could not read input file.");

        debug!(
            "attempting decryption with {} region keys",
            config_holder.region
        );

        match get_apimanager_keys(&config_holder.region) {
            Some(keys) => {
                debug!("Key len: {} IV len: {}", keys.0.len(), keys.1.len());
                let decrypted = decrypt_aes_cbc(&byte_buffer, keys.0, keys.1).unwrap();

                output_file
                    .write_all(&decrypted)
                    .expect("Could not write to output file.");
            }
            None => {
                error!(
                    "No valid decryption keys found for region {}",
                    config_holder.region
                );
                std::process::exit(1);
            }
        }
        return;
    } else if let Some(Command::AbInfoEncrypt(ref encrypt_options)) = opts.command {
        info!(
            "Encrypting assetbundle {} into {}",
            encrypt_options.decrypted_path.display(),
            encrypt_options.output.display()
        );

        let mut input_file =
            File::open(&encrypt_options.decrypted_path).expect("Could not read input file.");

        let mut output_file =
            File::create(&encrypt_options.output).expect("Could not open output file for writing.");

        let mut byte_buffer = Vec::new();
        input_file
            .read_to_end(&mut byte_buffer)
            .expect("Could not read input file.");

        debug!(
            "attempting encryption with {} region keys",
            config_holder.region
        );

        match get_apimanager_keys(&config_holder.region) {
            Some(keys) => {
                info!("Key len: {} IV len: {}", keys.0.len(), keys.1.len());
                let encrypted = encrypt_aes_cbc(&byte_buffer, keys.0, keys.1).unwrap();

                output_file
                    .write_all(&encrypted)
                    .expect("Could not write to output file.");
            }
            None => {
                error!(
                    "No valid decryption keys found for region {}",
                    config_holder.region
                );
                std::process::exit(1);
            }
        }
        return;
    } else if let Some(Command::GenAssetBundle(ref options)) = opts.command {
        info!(
            "Converting {} into an assetbundle!",
            options.assetbundle_path.display()
        );

        let modpack_data = fs::read_to_string(&options.assetbundle_path).unwrap_or_else(|_| {
            panic!(
                "Could not read {}! Please try redownloading mods and fixing permissions.",
                options.assetbundle_path.display()
            )
        });

        let mod_data: ModData = toml::from_str(&modpack_data).unwrap_or_else(|_| {
            panic!(
                "{} is not formatted properly! Check if MikuMikuLoader is out of date.",
                options.assetbundle_path.display()
            )
        });

        create_assetbundle(mod_data).unwrap();

        return;
    }

    let injector_config_path = Path::new(&config_holder.advanced.sekai_injector_config_path);

    let mut injector_file_contents = String::new();

    let mut injector_config_holder: sekai_injector::Config = match File::open(injector_config_path)
    {
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

    // In the case of the first run, we need to download the versions.json to proceed if it doesn't exist
    let versions_path = &format!("{}/versions.json", config_holder.advanced.assets.asset_path);
    let versions_file = Path::new(versions_path);

    if !versions_file.exists() {
        info!("Downloading versions file.");

        let client = reqwest::Client::new();

        let url = format!(
            "https://{}/versions.json",
            config_holder.advanced.assets.common_asset_url
        );

        debug!("grabbing versions file from {url}");
        let resp = client.get(url).send().await.unwrap();

        let asset_config = &config_holder.advanced.assets;

        match create_dir_all(asset_config.asset_path.to_string()) {
            Ok(_) => {}
            Err(e) => {
                error!(
                    "Cannot create {}: {e}. Trying to download anyway...",
                    asset_config.asset_path
                )
            }
        }

        let mut file = std::fs::File::create(format!("{}/versions.json", asset_config.asset_path))
            .expect("Failed to write asset to file");
        let mut content = Cursor::new(resp.bytes().await.unwrap());
        std::io::copy(&mut content, &mut file).expect("Failed to write asset to file");
    }

    // Read versions.json and update resource_prefix if it is necessary.
    let (_, data_version, asset_version) = match update_injection_appversion(
        &mut injector_config_holder,
        &config_holder,
    ) {
        Ok(version) => version,
        Err(e) => {
            let msg = &format!(
                "Could not update appversion and apphash prefix for assetbundle domain. It is likely injection will never trigger. Err: {e}"
            );
            error!("{msg}");
            notify_mml(msg);
            (String::from(""), String::from(""), String::from(""))
        }
    };

    match update_assets(&config_holder, data_version).await {
        Ok(_) => {}
        Err(e) => error!("Could not update assets: {e}\n"),
    };

    // Now that we have the asset version we can check if the user wants to reload the abinfo and exit
    if let Some(Command::ReloadAbInfo(ref _options)) = opts.command {
        info!(
            "Reloading assetbundle info hashes! This may trigger multiple redownloads within the game."
        );

        reload_assetbundle_info(&config_holder, &asset_version)
            .await
            .unwrap();
        return;
    }

    // Reload the assetbundle info cache TODO: Don't reload unless needed
    info!(
        "Reloading assetbundle info hashes! This may trigger multiple redownloads within the game."
    );

    reload_assetbundle_info(&config_holder, &asset_version)
        .await
        .unwrap();

    let mut sekai_injector_enabled = true;

    let domain_info = injector_config_holder.extract_domain_info();

    for path in domain_info.1.iter().chain(domain_info.2.iter()) {
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

    let mut injection_hashmap = load_injection_maps(&injector_config_holder);

    debug!("injection_hashmap: {injection_hashmap:?}");

    // For assetbundle info domain, set the resource path to contian the correct version and platform

    let assetbundle_info_path = &format!(
        "/api/version/{}/os/{}",
        asset_version, config_holder.platform
    );

    if let Some(inner_map) = injection_hashmap.get_mut(&config_holder.advanced.assetbundle_info_url)
        && let Some(entry) = inner_map.get_mut(assetbundle_info_path)
    {
        let asset_path = format!(
            "{}{}",
            config_holder.advanced.assets.asset_path, assetbundle_info_path
        );

        debug!("Setting asset_path to {asset_path}");
        entry.0 = asset_path;
    } else {
        warn!("Could not assign correct asset path for assetbundle info request path!");
    }

    debug!("injection_hashmap: {injection_hashmap:?}");

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

    notify_mml("MikuMikuLoader running at http://0.0.0.0:3939");

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

    for domain in config.domains.clone() {
        domains.insert(domain.address, vec![record_info.clone()]);
    }

    let conf = DConfig { bind, domains };
    info!("DNS server config: {conf:?}");
    conf
}

/// Sets the injection prefix in `sekai_injector::Config` based on `MMLConfig`.
///
/// Returns a tuple `(apphash, data version, asset version)`.
pub fn update_injection_appversion(
    injector_config: &mut sekai_injector::Config,
    mml_config: &MMLConfig,
) -> Result<(String, String, String)> {
    let versions_path = format!("{}/versions.json", mml_config.advanced.assets.asset_path);
    let version_file = fs::File::open(versions_path)?;

    let versions: Versions = serde_json::from_reader(version_file)?;

    if let Some(assetbundle_domain) = injector_config
        .domains
        .iter_mut()
        .find(|x| x.address == mml_config.advanced.assetbundle_url)
    {
        if assetbundle_domain.resource_prefix.is_some() {
            warn!("Prefix is manually set, will not override with up-to-date apphash and version!");
        } else {
            let prefix = format!("/{}/{}", versions.data_version, versions.asset_hash);
            debug!("Setting prefix to {prefix}");

            assetbundle_domain.resource_prefix = Some(prefix);
        }
    } else {
        anyhow::bail!(
            "Could not update required request prefixes, may be unable to succesfully inject resources!"
        );
    }

    if let Some(assetbundle_info_domain) = injector_config
        .domains
        .iter_mut()
        .find(|x| x.address == mml_config.advanced.assetbundle_info_url)
    {
        if assetbundle_info_domain.resource_prefix.is_some() {
            warn!("Prefix is manually set, will not override with up-to-date asset version!");
        } else {
            let prefix = format!(
                "/api/version/{}/os/{}",
                versions.asset_version, mml_config.platform
            );
            debug!("Setting prefix to {prefix}");

            assetbundle_info_domain.resource_prefix = Some(prefix);
        }
    } else {
        anyhow::bail!(
            "Could not update required request prefixes, may be unable to succesfully inject resources!"
        );
    }

    Ok((
        versions.app_hash,
        versions.data_version,
        versions.asset_version,
    ))
}

pub async fn update_assets(
    config: &MMLConfig,
    asset_version: String,
) -> Result<(), Box<dyn Error>> {
    // TODO: Implement retries
    let asset_config = &config.advanced.assets;

    let abinfo_url = vec![format!(
        "api/version/{}/os/{}",
        asset_version, config.platform
    )];

    notify_mml("Checking assets for updates...");

    let client = reqwest::Client::new();
    let mut tasks = FuturesUnordered::<Pin<Box<dyn Future<Output = String>>>>::new();

    for (list, base_url) in [
        &asset_config.needed_asset_files,
        &asset_config.needed_template_files,
        &asset_config.needed_live2d_files,
        &abinfo_url,
    ]
    .iter()
    .zip([
        &asset_config.common_asset_url,
        &asset_config.template_asset_url,
        &asset_config.live2d_asset_url,
        &config.advanced.assetbundle_info_url,
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
                warn!("Upstream server doesn't support etag, redownloading asset! Provided headers: {:?}", resp.headers());
            }

            let existing_etag_path = format!("{}/{asset}.etag", asset_config.asset_path);
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

                match create_dir_all(format!("{}/{asset_dir}", asset_config.asset_path)) {
                    Ok(_) => {}
                    Err(e) => {
                        error!(
                            "Cannot create {}/{asset_dir}: {e}. Trying to download anyway...", asset_config.asset_path
                        )
                    }
                }

                let mut file = std::fs::File::create(format!("{}/{asset}", asset_config.asset_path)).expect("Failed to write asset to file");
                let mut content = Cursor::new(resp.bytes().await.unwrap());
                std::io::copy(&mut content, &mut file).expect("Failed to write asset to file");
            }

            if etag_needs_update && new_etag_val.is_some() {
                // Update etag
                debug!("Storing etag for {asset}");

                match File::create(format!("{}/{asset}.etag", asset_config.asset_path)) {
                    Ok(mut etag_file) => match write!(etag_file, "{}", new_etag_val.unwrap()) {
                        Ok(_) => {}
                        Err(e) => {
                            error!(
                                "Failed to write {}/{asset}.etag! Asset will always be redownloaded. Err: {e}", asset_config.asset_path
                            )
                        }
                    },
                    Err(e) => error!(
                        "Failed to write {}/{asset}.etag! Asset will always be redownloaded. Err: {e}", asset_config.asset_path
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
