use anyhow::{ensure, Result};
use reqwest::blocking::{Client, Response};
use std::io::Read;
use std::time::Duration;

pub fn client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent(concat!(
            "osrs-random/",
            env!("CARGO_PKG_VERSION"),
            " (https://github.com/stackrot/osrs-random)"
        ))
        .https_only(true)
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .build()?)
}

pub fn read_response(response: Response, limit: u64) -> Result<Vec<u8>> {
    let response = response.error_for_status()?;
    let mut bytes = Vec::new();
    response.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "Response exceeds the size limit"
    );
    Ok(bytes)
}
