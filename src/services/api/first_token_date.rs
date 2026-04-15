// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/firstTokenDate.ts
//! First token date module
//! Fetches and caches the user's first Claude Code token date

use std::collections::HashMap;

/// Get OAuth config
fn get_oauth_config() -> crate::constants::oauth::OauthConfig {
    crate::constants::oauth::get_oauth_config()
}

/// Get auth headers from HTTP utils
fn get_auth_headers() -> crate::utils::http::AuthHeaders {
    crate::utils::http::get_auth_headers()
}

/// Get global config from config utils
fn get_global_config() -> crate::utils::config::GlobalConfig {
    crate::utils::config::get_global_config()
}

/// Save global config
fn save_global_config(update: impl FnOnce(&mut crate::utils::config::GlobalConfig)) {
    let mut config = get_global_config();
    update(&mut config);
    let _ = crate::utils::config::save_global_config(&config);
}

/// Get Claude Code user agent
fn get_claude_code_user_agent() -> String {
    format!("ai-agent/{}", env!("CARGO_PKG_VERSION"))
}

/// Fetch the user's first Claude Code token date and store in config.
/// This is called after successful login to cache when they started using Claude Code.
pub async fn fetch_and_store_claude_code_first_token_date() -> Result<(), String> {
    let config = get_global_config();

    if config.claude_code_first_token_date.is_some() {
        return Ok(());
    }

    let auth_headers = get_auth_headers();
    if let Some(error) = auth_headers.error {
        log::error!("Failed to get auth headers: {}", error);
        return Ok(());
    }

    let oauth_config = get_oauth_config();
    let url = format!(
        "{}/api/organization/claude_code_first_token_date",
        oauth_config.base_api_url
    );

    let mut headers = auth_headers.headers;
    headers.insert("User-Agent".to_string(), get_claude_code_user_agent());

    let reqwest_headers: reqwest::header::HeaderMap = headers
        .into_iter()
        .filter_map(|(k, v)| {
            let key: reqwest::header::HeaderName = k.parse().ok()?;
            let value: reqwest::header::HeaderValue = v.parse().ok()?;
            Some((key, value))
        })
        .collect();

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(10000))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to build HTTP client: {}", e);
            return Ok(());
        }
    };

    let response = match client.get(&url).headers(reqwest_headers).send().await {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to fetch first token date: {}", e);
            return Ok(());
        }
    };

    let data: serde_json::Value = match response.json().await {
        Ok(d) => d,
        Err(e) => {
            log::error!("Failed to parse first token date response: {}", e);
            return Ok(());
        }
    };

    let first_token_date = data.get("first_token_date").and_then(|v| v.as_str()).map(String::from);

    // Validate the date if it's not null
    if let Some(ref date_str) = first_token_date {
        if chrono::DateTime::parse_from_rfc3339(date_str).is_err() {
            log::error!(
                "Received invalid first_token_date from API: {}",
                date_str
            );
            // Don't save invalid dates
            return Ok(());
        }
    }

    save_global_config(|cfg| {
        cfg.claude_code_first_token_date = first_token_date;
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_first_token_date_stub() {
        // This will return early since config already has the value
        let result = fetch_and_store_claude_code_first_token_date().await;
        assert!(result.is_ok());
    }
}