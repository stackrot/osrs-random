use anyhow::{bail, ensure, Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read, Write};
use std::time::Duration;

const RELEASE_URL: &str = "https://api.github.com/repos/stackrot/osrs-random/releases/latest";
const DOWNLOAD_URL: &str = "https://github.com/stackrot/osrs-random/releases/download/";
const MAX_ASSET_SIZE: u64 = 100 * 1024 * 1024;
pub const RELEASE_TAG: Option<&str> = option_env!("OSRS_RANDOM_RELEASE_TAG");

#[derive(Debug, Deserialize)]
pub struct Release {
    pub tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    size: u64,
    digest: Option<String>,
}

pub fn available() -> Result<Option<Release>> {
    let client = crate::net::client()?;
    let response = client
        .get(RELEASE_URL)
        .header("Accept", "application/vnd.github+json")
        .send()?;
    let bytes = crate::net::read_response(response, 1024 * 1024)?;
    let release: Release = serde_json::from_slice(&bytes)?;
    ensure!(
        !release.draft && !release.prerelease,
        "Latest release is not stable"
    );
    let current = RELEASE_TAG.unwrap_or(env!("CARGO_PKG_VERSION"));
    if is_newer(current, &release.tag_name)? {
        Ok(Some(release))
    } else {
        Ok(None)
    }
}

fn is_newer(current: &str, latest: &str) -> Result<bool> {
    let current = current.strip_prefix('v').unwrap_or(current);
    let latest = latest.strip_prefix('v').unwrap_or(latest);
    let timestamp = |tag: &str| tag.len() == 14 && tag.bytes().all(|c| c.is_ascii_digit());
    match (timestamp(current), timestamp(latest)) {
        (true, true) => Ok(latest > current),
        (false, false) => {
            let current = semver::Version::parse(current).context("Invalid installed version")?;
            let latest = semver::Version::parse(latest).context("Invalid release version")?;
            ensure!(latest.pre.is_empty(), "Latest release is a prerelease");
            Ok(latest.cmp_precedence(&current).is_gt())
        }
        _ => bail!("Cannot compare this build with the release tag; install an official release from https://github.com/stackrot/osrs-random/releases"),
    }
}

fn platform(target: &str) -> Result<(&'static str, &'static str)> {
    match target {
        "x86_64-unknown-linux-gnu" => Ok(("osrs-random-linux.zip", "osrs-random")),
        "x86_64-pc-windows-msvc" => Ok(("osrs-random-windows.zip", "osrs-random.exe")),
        _ => bail!("Self-updating is unavailable for {target}; build from source"),
    }
}

pub fn install(release: &Release) -> Result<()> {
    let client = crate::net::client()?;
    let (asset_name, binary_name) = platform(env!("OSRS_RANDOM_TARGET"))?;
    let binary = prepare(&client, release, asset_name, binary_name)?;
    self_replace::self_replace(binary.path())
        .context("Could not replace the executable; check its directory permissions")?;
    println!(
        "Installed {}. Restart osrs-random to use it.",
        release.tag_name
    );
    Ok(())
}

fn prepare(
    client: &Client,
    release: &Release,
    asset_name: &str,
    binary_name: &str,
) -> Result<tempfile::NamedTempFile> {
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .context("Release has no download for this platform")?;
    validate_asset(asset, &release.tag_name)?;
    let response = client
        .get(&asset.browser_download_url)
        .timeout(Duration::from_secs(120))
        .send()?;
    let bytes = crate::net::read_response(response, MAX_ASSET_SIZE)?;
    verify_download(asset, &bytes)?;
    extract_binary(&bytes, binary_name)
}

fn validate_asset(asset: &Asset, tag: &str) -> Result<()> {
    ensure!(
        asset.size > 0 && asset.size <= MAX_ASSET_SIZE,
        "Invalid release asset size"
    );
    let expected = format!("{DOWNLOAD_URL}{tag}/{}", asset.name);
    ensure!(
        asset.browser_download_url == expected,
        "Unexpected release download URL"
    );
    let digest = asset
        .digest
        .as_deref()
        .and_then(|value| value.strip_prefix("sha256:"))
        .context("Release asset has no SHA-256 digest")?;
    ensure!(
        digest.len() == 64 && digest.bytes().all(|c| c.is_ascii_hexdigit()),
        "Invalid release SHA-256 digest"
    );
    Ok(())
}

fn checksum(bytes: &[u8]) -> String {
    let hex: String = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("sha256:{hex}")
}

fn verify_download(asset: &Asset, bytes: &[u8]) -> Result<()> {
    ensure!(
        bytes.len() as u64 == asset.size,
        "Incomplete release download"
    );
    let digest = checksum(bytes);
    ensure!(
        asset
            .digest
            .as_deref()
            .is_some_and(|expected| expected.eq_ignore_ascii_case(&digest)),
        "Release download failed SHA-256 verification"
    );
    Ok(())
}

fn extract_binary(bytes: &[u8], name: &str) -> Result<tempfile::NamedTempFile> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    let entry = archive
        .by_name(name)
        .context("Release archive is missing the executable")?;
    ensure!(
        entry.is_file() && !entry.is_symlink(),
        "Invalid executable in release archive"
    );
    let size = entry.size();
    ensure!(
        size > 0 && size <= MAX_ASSET_SIZE,
        "Invalid executable size"
    );
    let mut binary = tempfile::NamedTempFile::new()?;
    let copied = std::io::copy(&mut entry.take(MAX_ASSET_SIZE + 1), &mut binary)?;
    ensure!(copied == size, "Incomplete executable in release archive");
    binary.flush()?;
    Ok(binary)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(bytes: &[u8]) -> Asset {
        Asset {
            name: "osrs-random-linux.zip".into(),
            browser_download_url: format!("{DOWNLOAD_URL}v1.2.0/osrs-random-linux.zip"),
            size: bytes.len() as u64,
            digest: Some(checksum(bytes)),
        }
    }

    #[test]
    fn compares_semantic_versions_without_downgrading() {
        assert!(is_newer("1.9.0", "v1.10.0").unwrap());
        assert!(!is_newer("v1.10.0", "1.9.0").unwrap());
        assert!(!is_newer("1.2.0", "v1.2.0").unwrap());
        assert!(!is_newer("1.2.0+local", "v1.2.0+release").unwrap());
        assert!(is_newer("1.2.0", "v1.3.0-beta.1").is_err());
    }

    #[test]
    fn compares_timestamp_releases_without_downgrading() {
        assert!(is_newer("v20260704150523", "v20260912090000").unwrap());
        assert!(!is_newer("v20260912090000", "v20260704150523").unwrap());
        assert!(!is_newer("v20260912090000", "v20260912090000").unwrap());
        assert!(is_newer("1.1.0", "v20260912090000").is_err());
        assert!(is_newer("1.1.0", "invalid").is_err());
    }

    #[test]
    fn rejects_unsupported_architectures_and_abis() {
        assert!(platform("x86_64-unknown-linux-gnu").is_ok());
        assert!(platform("x86_64-pc-windows-msvc").is_ok());
        assert!(platform("aarch64-unknown-linux-gnu").is_err());
        assert!(platform("x86_64-unknown-linux-musl").is_err());
        assert!(platform("x86_64-pc-windows-gnu").is_err());
    }

    #[test]
    fn rejects_missing_digests_foreign_urls_and_tampered_downloads() {
        let mut asset = asset(b"archive");
        validate_asset(&asset, "v1.2.0").unwrap();
        verify_download(&asset, b"archive").unwrap();
        assert!(verify_download(&asset, b"changed").is_err());
        assert!(verify_download(&asset, b"short").is_err());
        asset.browser_download_url = "https://example.com/archive.zip".into();
        assert!(validate_asset(&asset, "v1.2.0").is_err());
        asset.digest = None;
        assert!(verify_download(&asset, b"archive").is_err());
    }

    #[test]
    fn extracts_only_the_expected_binary() {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        writer.start_file("../outside", options).unwrap();
        writer.write_all(b"unwanted").unwrap();
        writer.start_file("osrs-random", options).unwrap();
        writer.write_all(b"executable").unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        let binary = extract_binary(&bytes, "osrs-random").unwrap();
        assert_eq!(std::fs::read(binary.path()).unwrap(), b"executable");
        assert!(extract_binary(&bytes, "osrs-random.exe").is_err());
        assert!(extract_binary(b"broken", "osrs-random").is_err());
    }
}
