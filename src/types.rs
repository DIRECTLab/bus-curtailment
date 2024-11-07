use serde::{Deserialize, Serialize};
use chrono::{DateTime, Duration, Local, Timelike, Utc};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MeterValue {
    pub connector_id: i32,
    pub charger_id: String,
    pub transaction_id: i32,
    pub time_stamp: DateTime<Utc>,
    pub sampled_value: serde_json::Value
}

pub struct Soc {
    pub charger_id: String,
    pub value: f32
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Charger {
    pub id: String,
    pub charger_name: String,
    pub location_id: Option<i32>, 
    pub communicate_through: CommunicationType,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CommunicationType {
    RustDirectOcpp,
    OpenAdrMicrogrid
}


#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub connector_id:    i32,
    pub id_tag:          String,
    pub meter_start:     i32,
    pub timestamp_start: DateTime<Utc>,
    pub transaction_id:  Option<i32>,
    pub meter_stop:      Option<i32>,
    pub timestamp_stop:  Option<DateTime<Utc>>,
    pub stop_reason:     Option<String>,
    pub charger_id:      Option<String>
}
