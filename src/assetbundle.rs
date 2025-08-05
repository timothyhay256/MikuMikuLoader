use std::path::Path;

// Credit for reverse engineering and decryption method of assetbundle info goes to https://github.com/mos9527/sssekai
use aes::Aes128;
use anyhow::Result;
use block_padding::{Padding, Pkcs7};
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use chrono::{Datelike, Local, Timelike};
use log::{debug, error, warn};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use walkdir::WalkDir;

use crate::{
    mods::{CacheInvalidDuration, ModData},
    utils::{ABInfoRoot, Config},
};

type Aes128CbcEnc = cbc::Encryptor<Aes128>;
type Aes128CbcDec = cbc::Decryptor<Aes128>;

/// Returns an tuple containing region specific keys used for assetbundle info decryption
/// All credits to the source of the keys goes to https://github.com/mos9527/sssekai
pub fn get_apimanager_keys(lang: &str) -> Option<(&'static [u8], &'static [u8])> {
    match lang {
        "en" => Some((
            b"\xdf\x38\x42\x14\xb2\x9a\x3a\xdf\xbf\x1b\xd9\xee\x5b\x16\xf8\x84",
            b"~\x85l\x90y\x87\xf8\xae\xc6\xaf\xc0\xc5G8\xfc~",
        )),
        "jp" | "tw" | "kr" | "cn" => Some((b"g2fcC0ZczN9MTJ61", b"msx3IV0i9XE5uYZ1")),
        _ => None,
    }
}

pub fn encrypt_aes_cbc(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; data.len() + 16]; // Allocate room for padding
    let cipher = Aes128CbcEnc::new_from_slices(key, iv).unwrap();
    let out = cipher
        .encrypt_padded_b2b_mut::<Pkcs7>(data, &mut buf)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
    Ok(out.to_vec())
}

pub fn decrypt_aes_cbc(encrypted: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; encrypted.len()];
    let cipher = Aes128CbcDec::new_from_slices(key, iv).unwrap();
    let out = cipher
        .decrypt_padded_b2b_mut::<Pkcs7>(encrypted, &mut buf)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    Ok(out.to_vec())
}

/// Invalidate caches due for invalidation.
/// Uses the string FakePlaceholderHashYYYYMMDDSSMS
/// in case of permanently invalid hash and
/// FakePlaceholderHash0000000000000
/// as the "new" hash
pub async fn reload_assetbundle_info(config: Config, asset_version: String) -> Result<()> {
    let mod_path = Path::new("mods");

    let mut assetbundle_info = File::open(&format!(
        "{}/api/version/{}/os/{}",
        config.advanced.assets.asset_path, asset_version, config.platform
    ))
    .await?;

    let mut byte_buffer = Vec::new();
    assetbundle_info.read_to_end(&mut byte_buffer).await?;

    let key = get_apimanager_keys(&config.region).unwrap();
    let decrypted_abinfo = decrypt_aes_cbc(&byte_buffer, key.0, key.1)?;

    let mut abinfo: ABInfoRoot = rmp_serde::from_slice(&decrypted_abinfo)?;

    // Loop through and modify abinfo to have an invalid hash for each asset
    for entry in WalkDir::new(mod_path) {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file()
                    && entry.path().extension().and_then(|e| e.to_str()) == Some("toml")
                {
                    let entry_data = tokio::fs::read_to_string(entry.path()).await.unwrap_or_else(|_| {
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

                    for asset in mod_data.invalidated_assets {
                        debug!("Invalidating cache for {}", asset.resource_path);
                        match abinfo.bundles.get_mut(&asset.resource_path) {
                            Some(bundle) => match asset.duration {
                                CacheInvalidDuration::PermanentlyInvalid => {
                                    let now = Local::now();

                                    let formatted = format!(
                                        "{:04}{:02}{:02}{:02}{:03}",
                                        now.year(),
                                        now.month(),
                                        now.day(),
                                        now.second(),
                                        now.timestamp_subsec_millis()
                                    );

                                    bundle.hash = format!("FakePlaceholderHash{formatted}");
                                }
                                CacheInvalidDuration::InitiallyInvalid => {
                                    bundle.hash = "FakePlaceholderHash0000000000000".to_string(); // TODO: Track if this asset has already been injected
                                }
                            },
                            None => warn!(
                                "No matching ABInfo asset found for asset {}, it's cache will not be invalidated",
                                asset.resource_path
                            ),
                        }
                    }
                }
            }
            Err(ref e) => {
                error!("Could not read {entry:?}, skipping scan. Err: {e}");
            }
        }
    }

    let abinfo_string = rmp_serde::to_vec(&abinfo)?;

    let encrypted_abinfo = encrypt_aes_cbc(&abinfo_string, key.0, key.1)?;

    // Recreate the assetbundle info with newly invalid hashes
    let mut assetbundle_info = File::create(&format!(
        "{}/api/version/{}/os/{}",
        config.advanced.assets.asset_path, asset_version, config.platform
    ))
    .await?;
    assetbundle_info.write_all(&encrypted_abinfo).await?;

    Ok(())
}
