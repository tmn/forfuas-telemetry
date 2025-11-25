use rumqttc::{AsyncClient, EventLoop, MqttOptions};

use callsign::CallsignService;

mod buffer;
mod config;
mod http_client;
mod telemetry;
mod telemetry_service;

use crate::config::Config;
use crate::http_client::HttpClient;
use crate::telemetry_service::TelemetryService;

fn create_connection(config: &Config) -> (AsyncClient, EventLoop) {
    let mut mqttoptions = MqttOptions::new("rumqtt-async", &config.mqtt_broker, config.mqtt_port);
    mqttoptions.set_credentials(&config.mqtt_username, &config.mqtt_password);

    AsyncClient::new(mqttoptions, 10)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config = Config::from_env().expect("Failed to load config from environment");

    let (mqtt_client, eventloop) = create_connection(&config);
    let http_client = HttpClient::new(&config).expect("Failed to create HTTP client");

    let callsign_service = CallsignService::new("data/callsigns.db").expect("Callsigns db not available.");

    let mut service = TelemetryService::new(http_client, mqtt_client, eventloop, callsign_service);
    service
        .subscribe()
        .await
        .expect("Failed to subscribe to MQTT topics");
    service.run().await;
}
