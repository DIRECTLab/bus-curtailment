use reqwest::{Error, Client};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

async fn get_chargers(client: &Client, req_url: &String, location_id: i32 ) -> Result<Vec<Charger>, Error> {
    /*
     * Get all chargers and parse their output to find chargers with the desired location id
     **/
    let mut charger_url_path: String = req_url.clone();
    charger_url_path.push_str("/data/chargers");

    let res = client.get(&charger_url_path).send().await?;
    let body = res.text().await?;
    let chargers: Vec<Charger> = serde_json::from_str(&body).unwrap();
    let only_relevant_chargers: Vec<Charger> = chargers
        .into_iter()
        .filter(|charger| {
            if let Some(charger_location_id) = charger.location_id{
                if charger_location_id == location_id { 
                    return true;
                }
            } 
            return false
        })
        .collect();
    Ok(only_relevant_chargers)
}

async fn get_meter_values(client: &Client, req_url: &String, chargers: Vec<Charger>) -> Result<Vec<MeterValue>, Error>{
    /*
     * given a list of chargers, return the most meter values for all connectors from the charger
     */

    let mut meter_values: Vec<MeterValue> = Vec::new();

    let mut metervalues_url_path: String = req_url.clone();
    metervalues_url_path.push_str("/data/meter-values");
    for charger in chargers {
        
        let res = client
        .get(&metervalues_url_path)
        .body::<Json>(

        json!({
            "charger_id": charger.id,
            "descending": true,
            "limit": 1
        }).into()
            )

        .send()
        .await?;
        
        let res_body = res.text().await?;

        let meter_val: MeterValue = serde_json::from_str(&res_body).unwrap();
        meter_values.push(meter_val);
    };
    Ok(meter_values)
}


async fn create_charge_profile() {
    /*
     * Create and send a charge profile to chargerhub which will
     * act on curtailment schedule. This charging profile should
     * have multiple rates throughout the transaciton to curtail
     * the entire charge. This is to say place all charging behavior
     * in the same profile instead of creating a new profile for
     * every change in behavior
     */
}

async fn get_charge_rate() {
    /*
     * Given a bus' current state of charge, determine the rate of charge 
     * needed to charge the bus by the desired time
     *
     * charge_rate = (((desired_soc - curr_soc) / 100) * capacity) / time_allotment
     *             = required concurrent charge rate to charge battery to desired SOC
     *               in desired timeframe.
     */
}

async fn assign_charge_rates() {
    /*
     * Iterate through time steps and determine how much to charge each vehicle
     * at a given timestep. These values will then be aggregated into charge profiles
     * to be sent to the charger. Should iterate backwards from end time in attempt to
     * push all charging to off-peak time. Ideally, this will account for cost of power
     * at each time step to determine optimal charge rate. If t_0 is reached and the
     * busses are still not at desired SOC, find cheapest points to increase charge rate
     * until desired SOC met.
     */
}


async fn create_charging_strategy() {
    /*
     * A loop which recalculates charging schedule when a bus is connected/disconnected
     * from a charger. This calculate the power needed across the whole station, determine
     * charge rates at each time step, and send charge profiles which reflect the charging plan
     */
}

#[tokio::main]
async fn main() -> Result<(), Error>{
    // obtain all environment variables
    let chargerhub_url = dotenv::var("CHARGERHUB_URL")
        .expect("CHARGERHUB_URL was not specified in .env")
        .parse::<String>()
        .expect("Something went catastrophically wrong with parsing the chargerhub URL.");

    let battery_capacity = dotenv::var("BATTERY_CAPACITY")
        .expect("BATTERY_CAPACITY was not specified in .env")
        .parse::<i32>()
        .expect("Something went wrong reading in the battery capacity. Please verify BATTERY_CAPACITY is of type i32");

    let peak_upper_bound = dotenv::var("PEAK_UPPER_BOUND")
        .expect("PEAK_UPPER_BOUND was not specified in .env")
        .parse::<i32>()
        .expect("Something went wrong reading in the battery capacity. Please verify PEAK_UPPER_BOUND is of type i32");

    let client = Client::new();

    let chargers = get_chargers(&client, &chargerhub_url, 2).await?;

    let meter_values = get_meter_values(&client, &chargerhub_url, chargers).await?;
    println!("{:#?}", meter_values);
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
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
