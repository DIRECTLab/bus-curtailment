use reqwest::Client;
use chrono::Utc;
use serde_json::json;
use crate::types::{ChargingBounds, Charger};

pub async fn create_charge_profile(
    client: &Client, 
    req_url: &str, 
    connector_id: &i32, 
    charger_id: &String, 
    charge_rate: &mut f32, 
    verbose_mode: &bool,
    crg_bounds:ChargingBounds ) 
{
    /*
     * Create and send a charge profile to chargerhub which will
     * act on curtailment schedule. This charging profile should
     * have multiple rates throughout the transaciton to curtail
     * the entire charge. This is to say place all charging behavior
     * in the same profile instead of creating a new profile for
     * every change in behavior
     */
    
    
    /*
    connector_id: u32,
    duration: Option<u32>,
    purpose: Option<ChargingProfilePurposeType>,
    stack_level: Option<u32>,
    transaction_id: Option<u64>,
    charge_rates: Vec<f32>,
    start_periods: Option<Vec<u32>>,
    start_schedule: Option<DateTime<Utc>>
    */
    
    // clamp charge rate between upper and lower bound
    if *charge_rate < crg_bounds.lower_bnd as f32 {
        *charge_rate = crg_bounds.lower_bnd as f32;
    }

    if *charge_rate > crg_bounds.upper_bnd as f32{
        *charge_rate = crg_bounds.upper_bnd as f32;
    }


    // We need to get the connector id from the transaction

    let mut url: String = req_url.to_owned();
        url.push_str(&format!("/command/{}/set-charge-profile", charger_id));

    let charge_profile = &json!({
                "connector_id": connector_id,
                "start_periods": [0],
                "stack_level": 0,
                "charge_rates": [charge_rate],
                "purpose": "TxDefaultProfile",
                "start_schedule": Utc::now(),
            });

    if *verbose_mode{
        println!("charge profile created: {}", charge_profile);
    }

    let _res = client
        .post(url)
        .json(charge_profile)
        .send()
        .await;
}


pub async fn add_soc_to_metervals(charger: Charger) {

}
