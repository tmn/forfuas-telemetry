use reqwest::Client;
use std::time::Duration;

use crate::config::Config;
use crate::telemetry::UavStatus;

pub struct HttpClient {
    client: Client,

    api_enabled: bool,
    api_host: String,
    api_key: String,
}

impl HttpClient {
    pub fn new(config: &Config) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(10)
            .build()?;

        Ok(HttpClient {
            client,
            api_enabled: config.api_enabled,
            api_host: config.api_host.clone(),
            api_key: config.api_key.clone(),
        })
    }

    pub async fn send_batch(&self, statuses: &[UavStatus]) {
        if !self.api_enabled {
            return;
        }

        match self
            .client
            .post(&format!("{}/v1/uav", self.api_host))
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .json(&statuses)
            .send()
            .await
        {
            Ok(response) => {
                if !response.status().is_success() {
                    eprintln!(
                        "API error: HTTP {} - {}",
                        response.status(),
                        response.status().canonical_reason().unwrap_or("Unknown")
                    );
                }
            }

            Err(e) => {
                eprintln!("Network error sending telemetry: {e:?}");
            }
        }
    }
}
