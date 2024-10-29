use reqwest::{Error, Client, header::{HeaderValue, CONTENT_TYPE}};
use chrono::{DateTime, Duration, Local, Timelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time;

async fn get_chargers(client: &Client, req_url: &String, location_id: i32, verbose_mode: &bool) -> Result<Vec<Charger>, Error> {
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
            false
        })
        .collect();
    if *verbose_mode {
        println!("{:#?}", only_relevant_chargers);
    }
    Ok(only_relevant_chargers)
}


async fn get_meter_values(client: &Client, req_url: &String, chargers: Vec<Charger>, verbose_mode: &bool) -> Result<Vec<MeterValue>, Error>{
    /*
     * given a list of chargers, return the most meter values for all connectors from the charger
     */

    let mut meter_values: Vec<MeterValue> = Vec::new();

    let mut metervalues_url_path: String = req_url.to_owned();
    metervalues_url_path.push_str("/data/meter-values");
    for charger in chargers {
        
        let res = client
            .get(&metervalues_url_path)
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(
                json!({
                    "charger_id": charger.id,
                    "descending": true,
                    "limit": 1
                }).to_string()
            )
            .send()
        .await?;
        
        let res_body = res.text().await?;

        let meter_val: Vec<MeterValue> = serde_json::from_str(&res_body).unwrap();
        meter_values.push(meter_val[0].clone());
    };

    if *verbose_mode {
        println!("{:#?}", meter_values);
    }

    Ok(meter_values)
}

fn parse_meterval(metervalue: &MeterValue) -> i8{
    /*
     * Given a metervalue, parse out the transaction ID and 
     * SOC metric.
     */

    let meterval = metervalue.sampled_value
        .as_array()
        .expect("Meter value is not of expected type (&Vec<Value>)")
        .iter()
        .find(|value| value["measurand"] == "SoC")        
        .expect("Unable to find state of charge information in meter value");

    String::from(meterval["value"].as_str().unwrap()).parse::<f32>().unwrap() as i8
}

async fn create_charge_profile(client: &Client, req_url: &String, connector_id: &i32, charger_id: &String, charge_rate: f32, verbose_mode: &bool) {
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

    let mut url: String = req_url.to_owned();
        url.push_str(&format!("/data/{}/transactions", charger_id));

    let transaction_res = client.get(url);
    // We need to get the connector id from the transaction

    let mut url: String = req_url.clone();
        url.push_str("/command/set-charge-profile");

    let charge_profile = &json!({
                "connector_id": connector_id,
                "stack_level": 0,
                "charge_rates": charge_rate,
            });

    if *verbose_mode{
        println!("charge profile created: {}", charge_profile);
    }

    let res = client
        .post(url)
        .json(charge_profile);
}

// Time_allotment is just what it sounds like, 
// charge_amount is the amount of SoC that should be recovered at the end of the time allotment and
// should be out of 100
async fn get_charge_rate(time_allotment: Duration, charge_amount: i8, battery_capacity: &i32, verbose_mode: &bool) -> f32 {
    /*
     * Given a bus' current state of charge, determine the rate of charge 
     * needed to charge the bus by the desired time
     *
     * charge_rate = (((desired_soc - curr_soc) / 100) * capacity) / time_allotment
     *             = required concurrent charge rate to charge battery to desired SOC
     *               in desired timeframe.
     */

    let charge_rate = (charge_amount as f32 / 100.0) * (*battery_capacity as f32) / 
                           (time_allotment.num_hours() as f32 + 
                           (time_allotment.num_minutes() as f32 / 60.0));
    if *verbose_mode {
        println!("charge rate {} calculated for charging {}% over {} minutes", charge_rate, charge_amount, time_allotment);
    }
    return charge_rate;

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

async fn runner_loop(client: &Client, chargerhub_url: &String, battery_capacity: &i32, desired_soc: &i8, verbose_mode: &bool) {
    const TIME_BETWEEN_LOOPS: u64 = 5 * 60; // number of minutes to wait between loops
                                            
    let TIME_BETWEEN_RECALCULATIONS = Duration::new(90 * 60, 0).expect("Static duration failed to initialize"); // number of minutes to wait between recalculating
                                                                                                                                             // charge charge rates
                                                 
    let mut initial_calculation = false; // have the initial charge profiles been calculated?
                                               
    let mut last_recalculation = Local::now(); // last time new charge profiles were calculated
                                                                
    let start_time = Local::now() // only perform curtailment if after start time
        .with_hour(19)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();

    // Set time to stop curtailment to 5am. If before midnight, add day to chrono datetime
    let mut stop_time = Local::now()
        .with_hour(5)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();

    stop_time = if last_recalculation > stop_time { // move stop time forward by a day if
       stop_time + Duration::days(1)                // calculation made before midnight
    } else {
       stop_time
    };
    
    if *verbose_mode {
        println!("time between loops: {},\ntime between recalculations: {},\ncurtailment start time: {},\ncurtailment stop time: {}", &TIME_BETWEEN_LOOPS, &TIME_BETWEEN_RECALCULATIONS, &start_time, &stop_time);
    }

    /*
     * This will be the loop which actually performs the steps necessary to perform curtailment
     */
    loop {

        //Check that the current time is after the last route ends and that no conditions have been met to recalculate the charge rates
        let right_now = Local::now();
        let time_delta = right_now - last_recalculation;
        //Conditions to recalculate charges includes if the current time is after bus routes end for the day, and a bus being connected/disconnected from the pool.
        //Additionally, charge profiles should be recalculated every N minutes to ensure charging is completed by the desired time
        if !initial_calculation || time_delta >= TIME_BETWEEN_RECALCULATIONS && right_now >= start_time {
            initial_calculation = true; // Set to true since initial value calculated after this point
                                        
            last_recalculation = Local::now();

            //Obtain all chargers at the bus depo site. 
            let chargers = get_chargers(&client, chargerhub_url, 2, verbose_mode)
                .await
                .expect("Unable to grab chargers from charge site");

            //Grab meter values for each charger
            let meter_values = get_meter_values(&client, chargerhub_url, chargers, verbose_mode)
                .await
                .expect("Failed to obtain meter values from charger hub");
            //Create charge profiles
            for value in meter_values {
                
                //parse the SOC out of the meter values and get % charge needed to get to desired SOC
                let soc_needed = desired_soc - parse_meterval(&value);
                //Calculate the power needed for each bus and create a charge profile based on the needed power
                let time_to_charge = stop_time - right_now;
                let charge_rate = get_charge_rate(time_to_charge, soc_needed, battery_capacity, verbose_mode).await;

                //submit charge profiles to chargerhub which should handle the communication with the charger
                create_charge_profile(client, chargerhub_url, &value.connector_id, &value.charger_id, charge_rate, verbose_mode).await;
            }
        }
        else {
            println!("Conditions not met to recalculate new charge profiles.\nchecking again at {}", right_now + TIME_BETWEEN_RECALCULATIONS);
        }
        //Sleep for reasonable amount of time
        std::thread::sleep(time::Duration::from_secs(TIME_BETWEEN_LOOPS));
    }

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
        .expect("Something went wrong reading in the peak upper bound. Please verify PEAK_UPPER_BOUND is of type i32");
    
    let desired_soc = dotenv::var("DESIRED_SOC")
        .expect("DESIRED_SOC was not specified in .env")
        .parse::<i8>()
        .expect("Something went wrong reading in the desired SOC. Please verify DESIRED_SOC is of type i8");


    let verbose_mode = dotenv::var("VERBOSE_MODE")
        .expect("VERBOSE_MODE was not specified in .env")
        .parse::<bool>()
        .unwrap_or_else(|_| return false); // default to false if not specified

    let client = Client::new();

    runner_loop(&client, &chargerhub_url, &battery_capacity, &desired_soc, &verbose_mode).await;
    
    Ok(())
}

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
