use rumqttc::{Event, Packet};
use std::time::Duration;

use crate::buffer::TelemetryBuffer;
use crate::http_client::HttpClient;
use crate::telemetry::{OsdMessage, Telemetry, UavStatus, calculate_course};

pub struct TelemetryService {
    http_client: HttpClient,
    mqtt_client: rumqttc::AsyncClient,
    eventloop: rumqttc::EventLoop,
    buffer: TelemetryBuffer,
}

impl TelemetryService {
    pub fn new(
        http_client: HttpClient,
        mqtt_client: rumqttc::AsyncClient,
        eventloop: rumqttc::EventLoop,
    ) -> Self {
        TelemetryService {
            http_client,
            mqtt_client,
            eventloop,
            buffer: TelemetryBuffer::new(),
        }
    }

    pub async fn run(&mut self) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        let mut reconnect_delay = Duration::from_secs(1);

        loop {
            tokio::select! {
                event = self.eventloop.poll() => {

                    match &event {
                        Ok(Event::Incoming(Packet::Publish(packet))) => {
                            match serde_json::from_slice::<OsdMessage>(&packet.payload) {
                                Ok(msg) => {
                                    let telemetry: Telemetry = msg.into();
                                    self.buffer.update(telemetry);
                                }
                                Err(error) => {
                                    println!("Failed to parse telemetry: {:?}", error);

                                    if let Ok(message) = std::str::from_utf8(&packet.payload) {
                                        println!("Raw message: {}", message);
                                    }
                                }
                            }
                        }
                        Ok(_) => {
                            reconnect_delay = Duration::from_secs(1);
                        }
                        Err(error) => {
                            println!("MQTT connection error: {error:?}");
                            println!("Reconnecting in {reconnect_delay:?}");

                            tokio::time::sleep(reconnect_delay).await;
                            reconnect_delay = std::cmp::min(reconnect_delay * 2, Duration::from_secs(60));
                        }
                    }
                }

                _ = interval.tick() => {
                    self.batch_process().await;
                }
            }
        }
    }

    pub async fn subscribe(&mut self) -> Result<(), rumqttc::ClientError> {
        self.mqtt_client
            .subscribe("thing/product/+/osd", rumqttc::QoS::AtMostOnce)
            .await
    }

    pub async fn batch_process(&mut self) {
        self.buffer.cleanup_stale_uavs(Duration::from_secs(30));

        let statuses: Vec<UavStatus> = self
            .buffer
            .get_all_uavs()
            .map(|state| {
                let course = state
                    .previous
                    .as_ref()
                    .map(|prev| calculate_course(prev, &state.current));
                UavStatus::from_telemetry(&state.current, course)
            })
            .collect();

        if !statuses.is_empty() {
            println!("Sending {} UAV statuses", statuses.len());
            self.http_client.send_batch(&statuses).await;
        }
    }
}
