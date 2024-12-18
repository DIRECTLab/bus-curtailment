use reqwest::Client;
use chrono::{DateTime, Duration, Local, Timelike, Utc};
use std::{collections::HashMap, time};
use crate::{
    get_data::{get_charge_rate, get_chargers, get_meter_values}, send_data::create_charge_profile, types::{ChargeProfile, ChargingBounds}, util::parse_meterval
};

pub async fn runner_loop(client: &Client, chargerhub_url: &String, battery_capacity: &i32, desired_soc: &i8, verbose_mode: &bool, auth_key: &String) {

    let charge_clamp_lower = dotenv::var("CHARGE_CLAMP_LOWER")
        .expect("CHARGE_CLAMP_LOWER was not specified in .env")
        .parse::<i32>()
        .expect("Something went wrong reading in the lower bound for charge rates. Please verify CHARGE_CLAMP_LOWER is of type i32");

    let charge_clamp_upper = dotenv::var("CHARGE_CLAMP_UPPER")
        .expect("CHARGE_CLAMP_UPPER was not specified in .env")
        .parse::<i32>()
        .expect("Something went wrong reading in the lower bound for charge rates. Please verify CHARGE_CLAMP_LOWER is of type i32");

    let location_id = dotenv::var("LOCATION_ID")
        .expect("LOCATION_ID was not specified in .env")
        .parse::<i32>()
        .expect("Something went wrong reading in the location ID for relevant chargers. Please verify LOCATION_ID is of type u32");

    let default_charge_rate = dotenv::var("CHARGE_RATE_DEFAULT")
        .expect("CHARGE_RATE_DEFAULT was not specified in .env")
        .parse::<f32>()
        .expect("Something went wrong reading in the default charge rate. Please verify CHARGE_RATE_DEFAULT is of type f32");
    
    let curtailment_start_hour = dotenv::var("CURTAILMENT_START_HOUR")
        .expect("CURTAILMENT_START_HOUR was not specified in .env")
        .parse::<u32>()
        .expect("Something went wrong reading in the default charge rate. Please verify CURTAILMENT_START_HOUR is of type u16");

    let curtailment_stop_hour = dotenv::var("CURTAILMENT_STOP_HOUR")
        .expect("CURTAILMENT_STOP_HOUR was not specified in .env")
        .parse::<u32>()
        .expect("Something went wrong reading in the default charge rate. Please verify CURTAILMENT_STOP_HOUR is of type u16");



    const TIME_BETWEEN_LOOPS: u64 = 5 * 60; // number of minutes to wait between loops
                                            
    let time_between_recalculations = Duration::new(15 * 60, 0).expect("Static duration failed to initialize"); // number of minutes to wait between recalculating
                                                                                                                                             // charge charge rates
                                                
    let mut prev_profiles = HashMap::new();
                                               
    let mut last_recalculation = Local::now() - Duration::minutes(TIME_BETWEEN_LOOPS as i64); // last time new charge profiles were calculated
                                                                
    let mut start_time =  set_start_time(curtailment_start_hour);

    // Set time to stop curtailment to 5am. If before midnight, add day to chrono datetime
    let mut stop_time = set_stop_time(last_recalculation, curtailment_stop_hour);

    let mut right_now = Local::now();

    if *verbose_mode {
        println!("time between loops: {},\ntime between recalculations: {},\ncurtailment start time: {},\ncurtailment stop time: {},\ncurrent server time: {}", 
            &TIME_BETWEEN_LOOPS, 
            &time_between_recalculations, 
            &start_time, 
            &stop_time, 
            &right_now
        );
    }

    /*
     * This will be the loop which actually performs the steps necessary to perform curtailment
     */
    loop {

        //Check that the current time is after the last route ends and that no conditions have been met to recalculate the charge rates
        right_now = Local::now();
        let time_delta = right_now - last_recalculation;
        //Conditions to recalculate charges includes if the current time is after bus routes end for the day, and a bus being connected/disconnected from the pool.
        //Additionally, charge profiles should be recalculated every N minutes to ensure charging is completed by the desired time

        //Check that right now is within a day of the start time, otherwise reset the start/end times
        let start_delta = start_time - right_now;
        if start_delta.num_hours() >= 24 {
            last_recalculation = Local::now();
            start_time = set_start_time(curtailment_start_hour);
            stop_time = set_stop_time(last_recalculation, curtailment_stop_hour);
        }

        
        //TODO: Add check here for if new busses were connected/disconnected
        //We could also add rules here for charge behavior based on time of night
        //(IE, if check occurred during non-peak then increase charge rate)

        if time_delta >= time_between_recalculations && right_now >= start_time {


                                        
            last_recalculation = Local::now();

            //Obtain all chargers at the bus depo site. 
            let chargers = get_chargers(client, chargerhub_url, location_id, verbose_mode, auth_key)
                .await
                .expect("Unable to grab chargers from charge site");

            //Grab meter values for each charger
            let meter_values = get_meter_values(client, chargerhub_url, chargers, verbose_mode, auth_key)
                .await
                .expect("Failed to obtain meter values from charger hub");
            //Create charge profiles
            for value in meter_values {
                //parse the SOC out of the meter values and get % charge needed to get to desired SOC
                let current_soc = parse_meterval(&value).await;

                if current_soc != -1 {

                    let soc_needed = if current_soc >= *desired_soc {
                        0
                    }
                    else {
                        desired_soc - current_soc
                    };


                    //Calculate the power needed for each bus and create a charge profile based on the needed power
                    let time_to_charge = stop_time - right_now;
                    let mut charge_rate = get_charge_rate(time_to_charge, soc_needed, battery_capacity, verbose_mode).await;

                    //submit charge profiles to chargerhub which should handle the communication with the charger
                    let charge_profile = create_charge_profile(
                        client, 
                        chargerhub_url, 
                        &value.connector_id, 
                        &value.charger_id, 
                        &mut charge_rate, 
                        stop_time.with_timezone(&Utc),
                        verbose_mode, 
                        ChargingBounds{lower_bnd: charge_clamp_lower, upper_bnd: charge_clamp_upper},
                        auth_key
                        ).await;
                    prev_profiles.entry(format!("{} - {}", &value.charger_id, &value.connector_id))
                        .or_insert_with(Vec::new)
                        .push(charge_profile);

                }
                else {
                    let empty_vec: Vec<ChargeProfile> = Vec::new(); // Define a static empty vector
                    let profile_list = prev_profiles.get(&format!("{} - {}", &value.charger_id, &value.connector_id)).unwrap_or(&empty_vec);
                    if !profile_list.is_empty() {
                        let most_recent_profile = profile_list
                            .last()
                            .expect("Unable to unwrap charge profile from previous profiles hashmap");
                        create_charge_profile(
                        client,
                        chargerhub_url, 
                        &value.connector_id, 
                        &value.charger_id, 
                        &mut most_recent_profile.charge_rates[0].to_owned(), 
                        stop_time.with_timezone(&Utc),
                        verbose_mode, 
                        ChargingBounds{lower_bnd: charge_clamp_lower, upper_bnd: charge_clamp_upper},
                        auth_key).await;
                    }

                    else { 
                        create_charge_profile(
                            client,
                            chargerhub_url, 
                            &value.connector_id, 
                            &value.charger_id, 
                            &mut default_charge_rate.to_owned(),
                        stop_time.with_timezone(&Utc),
                        verbose_mode, 
                        ChargingBounds{lower_bnd: charge_clamp_lower, upper_bnd: charge_clamp_upper},
                        auth_key).await;
                    }
                }
            }
        }
        else {
            println!("Conditions not met to recalculate new charge profiles.\nchecking again at {}", right_now + time_between_recalculations);
        }
        //Sleep for reasonable amount of time
        std::thread::sleep(time::Duration::from_secs(TIME_BETWEEN_LOOPS));
    }

}


fn set_start_time(curtailment_start_hour: u32) -> DateTime<Local>{
    let start_time = Local::now() // only perform curtailment if after start time
        .with_hour(curtailment_start_hour)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();
    start_time
}

fn set_stop_time(last_recalculation: DateTime<Local>, curtailment_stop_hour: u32) -> DateTime<Local> {
    // Set time to stop curtailment to stop time specified in env. If before midnight, add day to chrono datetime
    let mut stop_time = Local::now()
        .with_hour(curtailment_stop_hour)
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
 
    stop_time
}
