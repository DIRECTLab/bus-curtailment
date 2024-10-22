use reqwest::{Error, Url};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


async fn get_chargers(location_id: i32, req_url: String) -> Result<Vec<Charger>, Error> {
    /*
     * Get all chargers and parse their output to find chargers with the desired location id
     **/
    let mut charger_url_path: String = req_url.clone();
    charger_url_path.push_str("/data/chargers");

    let res = reqwest::get(&charger_url_path).await?;
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

async fn get_meter_values() {
    /*
     * given a list of chargers, return the most meter values for all connectors from the charger
     */
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


    let chargers = get_chargers(2, chargerhub_url).await?;
    println!("{:#?}", chargers);
    Ok(())
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
