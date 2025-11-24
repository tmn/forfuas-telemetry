mod buffer;
mod config;
mod httpclient;
mod telemetry;

use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use serde_json;
use std::time::Duration;

use buffer::TelemetryBuffer;
use config::Config;
use httpclient::HttpClient;
use telemetry::{OsdMessage, Telemetry};

use crate::telemetry::{UavStatus, calculate_course};

fn create_connection(config: &Config) -> (AsyncClient, EventLoop) {
    let mut mqttoptions = MqttOptions::new("rumqtt-async", &config.mqtt_broker, config.mqtt_port);
    mqttoptions.set_credentials(&config.mqtt_username, &config.mqtt_password);

    AsyncClient::new(mqttoptions, 10)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config = Config::from_env().expect("Failed to load config from environment");

    let (client, mut eventloop) = create_connection(&config);
    let http_client = HttpClient::new(&config).expect("Failed to create HTTP client");
    let mut buffer = TelemetryBuffer::new();

    client
        .subscribe("thing/product/+/osd", QoS::AtMostOnce)
        .await
        .unwrap();

    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            event = eventloop.poll() => {

                match &event {
                    Ok(Event::Incoming(Packet::Publish(packet))) => {


                        match serde_json::from_slice::<OsdMessage>(&packet.payload) {
                            Ok(msg) => {
                                let telemetry: Telemetry = msg.into();
                                buffer.update(telemetry);
                            }
                            Err(error) => {
                                println!("Failed to parse telemetry: {:?}", error);

                                // Print raw message for debugging
                                // if let Ok(message) = std::str::from_utf8(&packet.payload) {
                                //     println!("Raw message: {}", message);
                                // }
                            }
                        }
                    }
                    Ok(_) => {
                        // println!("Event = {notif:?}");
                    }
                    Err(error) => {
                        println!("Error = {error:?}");
                        break;
                    }
                }
            }

            _ = interval.tick() => {
                buffer.cleanup_stale_uavs(Duration::from_secs(30));

                let statuses: Vec<UavStatus> = buffer.get_all_uavs()
                    .map(|state| {
                        let course = state.previous.as_ref().map(|prev| {
                            calculate_course(prev, &state.current)
                        });
                        UavStatus::from_telemetry(&state.current, course)
                    })
                    .collect();

                if !statuses.is_empty() {
                    println!("Sending {} UAV statuses", statuses.len());
                    http_client.send_batch(&statuses).await;
                }
            }
        }
    }
}
