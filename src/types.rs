use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MeterValue {
    pub connector_id: i32,
    pub charger_id: String,
    pub transaction_id: i32,
    pub time_stamp: DateTime<Utc>,
    pub sampled_value: serde_json::Value
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

pub struct ChargingBounds {
    pub lower_bnd: i32,
    pub upper_bnd: i32
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
    pub charger_id:      Option<String>,
    pub voided:          Option<bool>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChargeProfile {
    pub charger_id:     String,
    pub connector_id:   i32,
    pub start_periods:  [i32; 1],
    pub stack_level:    i32,
    pub charge_rates:   [f32; 1],
    pub purpose:        String,
    pub start_schedule:     DateTime<Utc>
}
