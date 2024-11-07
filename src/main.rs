mod run_loop;
mod get_data;
mod send_data;
mod util;
mod types;

use reqwest::{Error, Client, header::{HeaderValue, CONTENT_TYPE}};
use chrono::{DateTime, Duration, Local, Timelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time;
use crate::run_loop::runner_loop;


#[tokio::main]
async fn main() -> Result<(), Error>{
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
        .unwrap_or_else(|_| return false); 

    let client = Client::new();

    runner_loop(&client, &chargerhub_url, &battery_capacity, &desired_soc, &verbose_mode).await;
    
    Ok(())
}


