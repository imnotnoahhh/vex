use crate::config::{self, Settings};
use crate::error::{Result, VexError};
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;

pub fn client_for_settings(settings: &Settings, user_agent: &str) -> Result<Client> {
    build_client(settings, user_agent)
}

pub fn client_for_current_context(user_agent: &str) -> Result<Client> {
    let settings = config::load_effective_settings_for_current_dir()?;
    build_client(&settings, user_agent)
}

pub fn client_for_global_settings(user_agent: &str) -> Result<Client> {
    let settings = config::load_settings()?;
    build_client(&settings, user_agent)
}

pub fn get_json_in_current_context<T: DeserializeOwned>(url: &str, user_agent: &str) -> Result<T> {
    let response = client_for_current_context(user_agent)?
        .get(url)
        .send()
        .map_err(VexError::Network)?
        .error_for_status()
        .map_err(VexError::Network)?;

    response.json().map_err(VexError::Network)
}

pub fn get_text_in_current_context(url: &str, user_agent: &str) -> Result<String> {
    let response = client_for_current_context(user_agent)?
        .get(url)
        .send()
        .map_err(VexError::Network)?
        .error_for_status()
        .map_err(VexError::Network)?;

    response.text().map_err(VexError::Network)
}

fn build_client(settings: &Settings, user_agent: &str) -> Result<Client> {
    let mut builder = Client::builder()
        .connect_timeout(settings.network.connect_timeout)
        .timeout(settings.network.read_timeout)
        .redirect(reqwest::redirect::Policy::limited(
            settings.network.max_http_redirects,
        ))
        .user_agent(user_agent.to_string());

    if let Some(proxy) = &settings.network.proxy {
        let proxy = reqwest::Proxy::all(proxy)
            .map_err(|err| VexError::Config(format!("Invalid proxy configuration: {}", err)))?;
        builder = builder.proxy(proxy);
    }

    builder.build().map_err(VexError::Network)
}
