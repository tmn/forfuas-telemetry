use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Telemetry {
    pub sn: String,
    pub latitude: f64,
    pub longitude: f64,
    pub height: f64,
    pub elevation: f64,
    pub attitude_head: f64,
    pub horizontal_speed: f64,
    pub vertical_speed: f64,
}

impl From<OsdMessage> for Telemetry {
    fn from(msg: OsdMessage) -> Self {
        Self {
            sn: msg.data.sn,
            latitude: msg.data.host.latitude,
            longitude: msg.data.host.longitude,
            height: msg.data.host.height,
            elevation: msg.data.host.elevation,
            attitude_head: msg.data.host.attitude_head,
            horizontal_speed: msg.data.host.horizontal_speed,
            vertical_speed: msg.data.host.vertical_speed,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OsdMessage {
    data: OsdData,
}

#[derive(Debug, Deserialize)]
struct OsdData {
    host: OsdHost,
    sn: String,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct OsdHost {
    latitude: f64,
    longitude: f64,
    height: f64,
    elevation: f64,
    attitude_head: f64,
    horizontal_speed: f64,
    vertical_speed: f64,
}

#[derive(Debug, Serialize)]
pub struct UavStatus {
    id: String,
    call_sign: String,
    latitude: f64,
    longitude: f64,
    altitude: f64,

    status: String,

    course: f64,
    ground_speed: f64,
    vertical_rate: f64,
    last_update: f64,
}

impl UavStatus {
    pub fn from_telemetry(telemetry: &Telemetry, course: Option<f64>) -> Self {
        UavStatus {
            id: telemetry.sn.clone(),
            call_sign: String::from("NFS Asker&Baerum"),
            latitude: telemetry.latitude,
            longitude: telemetry.longitude,
            altitude: telemetry.height,
            status: Self::get_status(&telemetry.elevation).to_string(),
            course: course.unwrap_or(telemetry.attitude_head.abs()),
            ground_speed: telemetry.horizontal_speed,
            vertical_rate: telemetry.vertical_speed,
            last_update: chrono::Utc::now().timestamp_millis() as f64 / 1000.0,
        }
    }

    fn get_status(elevation: &f64) -> &str {
        if *elevation > 0.0 {
            "AIRBORNE"
        } else {
            "GROUNDED"
        }
    }
}

pub fn calculate_course(prev: &Telemetry, current: &Telemetry) -> f64 {
    let lat1 = prev.latitude.to_radians();
    let lon1 = prev.longitude.to_radians();
    let lat2 = current.latitude.to_radians();
    let lon2 = current.longitude.to_radians();

    let dlon = lon2 - lon1;
    let y: f64 = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();

    let bearing = y.atan2(x).to_degrees();

    (bearing + 360.0) % 360.0
}
