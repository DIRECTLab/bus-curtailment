use reqwest::{Error, Client, header::{HeaderValue, CONTENT_TYPE, AUTHORIZATION}};
use chrono::{Duration};
use serde_json::json;
use crate::{
    types::{Charger, MeterValue},
    util::is_meterval_active
};

pub async fn get_chargers(client: &Client, req_url: &str, location_id: i32, verbose_mode: &bool, auth_key: &String) -> Result<Vec<Charger>, Error> {
    /*
     * Get all chargers and parse their output to find chargers with the desired location id and an
     * active transaction
     **/
    let mut charger_url_path: String = req_url.to_owned();
    charger_url_path.push_str("/data/chargers");

    // Create a HeaderValue for the Authorization header
    let auth_header_value = HeaderValue::from_str(&format!("Bearer {}", auth_key))
        .map_err(|err| {
            eprintln!("Invalid header value: {}", err);
        }).unwrap();


    // Build the request
    let request = client
        .get(&charger_url_path)
        .header(AUTHORIZATION, auth_header_value)
        .header(CONTENT_TYPE, "application/json")
        .body("{}")
        .build()?;
    // Print headers before sending the request
    if *verbose_mode {
        println!("Headers Sent:");
        for (key, value) in request.headers().iter() {
            println!("{}: {:?}", key, value);
        }
    }
    // Send the request
    let res = client.execute(request).await?;
    // let res = client.get(&charger_url_path).send().await?;
    let body = res.text().await?;
    let chargers: Vec<Charger> = serde_json::from_str(&body).unwrap_or_else(|err|{
        eprintln!("unable to unwrap, {}\nError: {}", &body, err);
        Vec::new()
    });
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


pub async fn get_meter_values(client: &Client, req_url: &String, chargers: Vec<Charger>, verbose_mode: &bool, auth_key: &String) -> Result<Vec<MeterValue>, Error>{
    /*
     * given a list of chargers, return the most recent meter values for all connectors from the charger
     */

    let mut meter_values: Vec<MeterValue> = Vec::new();

    let mut metervalues_url_path: String = req_url.to_owned();
    metervalues_url_path.push_str("/data/meter-values");
    for charger in chargers {
        for connector in 1..3{ // for each connector
            println!("Checking connector {}", connector);

            let auth_header_value = HeaderValue::from_str(&format!("Bearer {}", auth_key))
                .map_err(|err| {
                    eprintln!("Invalid header value: {}", err);
                }).unwrap();




            let res = client
                    .get(&metervalues_url_path)
                    .header(AUTHORIZATION, auth_header_value)
                    .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
                    .body(
                        json!({
                            "charger_id": charger.id,
                            "descending": true,
                            "limit": 1, // one meter val for each connector
                            "connector_id": connector
                        }).to_string()
                    )
                    .send()
                    .await?;


                
                let res_body = res.text().await?;

                let meter_val: Vec<MeterValue> = serde_json::from_str(&res_body).unwrap();
                if is_meterval_active(req_url, client, &meter_val[0], verbose_mode, auth_key).await {
                    meter_values.push(meter_val[0].clone());
                }

                if *verbose_mode {
                    println!("{:#?}", meter_val);
                }
            };


        }
        
    Ok(meter_values)
}



pub async fn get_charge_rate(time_allotment: Duration, charge_amount: i8, battery_capacity: &i32, verbose_mode: &bool) -> f32 {
    /*
     * Given a bus' current state of charge, determine the rate of charge 
     * needed to charge the bus by the desired time
     *
     * charge_rate = (((desired_soc - curr_soc) / 100) * capacity) / time_allotment
     *             = required concurrent charge rate to charge battery to desired SOC
     *               in desired timeframe.
     * 
     * @Input:  time_allotment   - amount of time vehicle will have to charge
     *          charge_amount    - SoC deficit needed to be filled (IE: battery at 60%, want 80%,
     *                             deficit is 20%)
     *          battery_capacity - total capacity of the battery of the vehicle being passed in
     *          verbose_mode     - display debug statements if true
     *
     * @Output: Needed charge rate in KW
     */

    let charge_rate = (charge_amount as f32 / 100.0) * (*battery_capacity as f32) / 
                           (time_allotment.num_hours() as f32 + 
                           (time_allotment.num_minutes() as f32 / 60.0));
    if *verbose_mode {
        println!("charge rate {}Kw calculated for charging +{}% over {} minutes", charge_rate, charge_amount, time_allotment.num_minutes());
    }
    charge_rate * 1000.0
}




