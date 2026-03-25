use crate::error::{Result, VexError};
use crate::http;
use reqwest::blocking::Client;
use tracing::warn;

pub(in crate::tools::python::releases) fn create_github_client() -> Result<Client> {
    http::client_for_current_context("vex-version-manager")
}

pub(in crate::tools::python::releases) fn fetch_text_with_retry(
    client: &Client,
    url: &str,
) -> Result<String> {
    let settings = crate::config::load_effective_settings_for_current_dir()?;
    let mut attempts = 0;
    let max_attempts = settings.network.download_retries.max(1);

    loop {
        match client.get(url).send() {
            Ok(response) => match response.error_for_status() {
                Ok(ok_response) => match ok_response.text() {
                    Ok(text) => return Ok(text),
                    Err(err) => {
                        if attempts + 1 < max_attempts {
                            warn!(
                                "Python upstream text fetch failed (attempt {}/{}): {}",
                                attempts + 1,
                                max_attempts,
                                err
                            );
                            attempts += 1;
                            std::thread::sleep(settings.network.retry_base_delay);
                            continue;
                        }
                        return Err(VexError::Network(err));
                    }
                },
                Err(err) => {
                    if err
                        .status()
                        .map(|status| status.is_client_error())
                        .unwrap_or(false)
                    {
                        return Err(VexError::Network(err));
                    }
                    if attempts + 1 < max_attempts {
                        warn!(
                            "Python upstream request failed (attempt {}/{}): {}",
                            attempts + 1,
                            max_attempts,
                            err
                        );
                        attempts += 1;
                        std::thread::sleep(settings.network.retry_base_delay);
                        continue;
                    }
                    return Err(VexError::Network(err));
                }
            },
            Err(err) => {
                if attempts + 1 < max_attempts {
                    warn!(
                        "Python upstream request failed (attempt {}/{}): {}",
                        attempts + 1,
                        max_attempts,
                        err
                    );
                    attempts += 1;
                    std::thread::sleep(settings.network.retry_base_delay);
                    continue;
                }
                return Err(VexError::Network(err));
            }
        }
    }
}

pub(in crate::tools::python::releases) fn fetch_latest_release_tag() -> Result<String> {
    let client = create_github_client()?;
    let response = client
        .get("https://github.com/astral-sh/python-build-standalone/releases/latest")
        .send()?
        .error_for_status()?;
    let final_url = response.url().clone();
    let tag = final_url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .filter(|segment| !segment.is_empty())
        .ok_or_else(|| {
            VexError::Parse("Unable to determine python-build-standalone release tag".to_string())
        })?;
    Ok(tag.to_string())
}

pub(in crate::tools::python::releases) fn fetch_sha256sums(tag: &str) -> Result<String> {
    let client = create_github_client()?;
    let sha256_url = format!(
        "https://github.com/astral-sh/python-build-standalone/releases/download/{}/SHA256SUMS",
        tag
    );
    fetch_text_with_retry(&client, &sha256_url)
}
