use dotenv::dotenv;
use std::env;

pub struct Config {
    pub mqtt_broker: String,
    pub mqtt_port: u16,
    pub mqtt_username: String,
    pub mqtt_password: String,

    pub api_enabled: bool,
    pub api_key: String,
    pub api_host: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenv().ok();

        let api_enabled = env::var("API_ENABLED")
            .unwrap_or(String::from("false"))
            .to_lowercase()
            .eq("true");

        Ok(Config {
            mqtt_broker: env::var("MQTT_BROKER").map_err(|_| "MQTT_BROKER not set".to_string())?,
            mqtt_port: env::var("MQTT_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1883),
            mqtt_username: env::var("MQTT_USERNAME")
                .map_err(|_| "MQTT_USERNAME not set".to_string())?,
            mqtt_password: env::var("MQTT_PASSWORD")
                .map_err(|_| "MQTT_PASSWORD not set".to_string())?,
            api_enabled,
            api_key: if api_enabled {
                env::var("API_KEY").map_err(|_| "API_KEY not set".to_string())?
            } else {
                String::new()
            },
            api_host: if api_enabled {
                env::var("API_HOST").map_err(|_| "API_HOST not set")?
            } else {
                String::new()
            },
        })
    }
}
