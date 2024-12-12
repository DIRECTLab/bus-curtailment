use crate::types::{MeterValue, Transaction};
use reqwest::{Client, header::{HeaderValue, CONTENT_TYPE, AUTHORIZATION}};
use serde_json::json;

pub fn parse_meterval(metervalue: &MeterValue) -> i8 {
    /*
     * Given a metervalue, parse out the transaction ID and 
     * SOC metric.
     */

    let sampled_values = match metervalue.sampled_value.as_array() {
        Some(values) => values,
        None => return -1, 
    };

    let meterval = match sampled_values.iter().find(|value| value["measurand"] == "SoC") {
        Some(value) => value,
        None => return -1, 
    };

    let value_str = match meterval["value"].as_str() {
        Some(s) => s,
        None => return -1, 
    };

    match value_str.parse::<f32>() {
        Ok(parsed_value) => parsed_value as i8,
        Err(_) => -1, 
    }
}


pub async fn is_meterval_active(req_url: &String, client: &Client, metervalue: &MeterValue, verbose_mode: &bool, auth_key: &String) -> bool{
    /*
     * Is the meter value for a transaction which has not ended?
     * will check if stop time is not null and return true or false
     * accordingly.
     */

    let mut url: String = req_url.to_owned();
        url.push_str(&format!("/data/{}/transactions", metervalue.charger_id));

    let auth_header_value = HeaderValue::from_str(&format!("Bearer {}", auth_key))
        .map_err(|err| {
            eprintln!("Invalid header value: {}", err);
        }).unwrap();




    let res = client
        .get(url)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .header(AUTHORIZATION, auth_header_value)
        .body(
            json!({
                "limit": 1,
                "connector_id": metervalue.connector_id
            }).to_string()
        )
        .send()
        .await
        .expect("Unable to process response from server");

    let res_body = res.text().await.unwrap();
    let transaction_data: Vec<Transaction> = serde_json::from_str(&res_body).expect("Unable to deserialize JSON into ");
    if *verbose_mode {
        println!("{:#?}", transaction_data);
    }
    if transaction_data[0].stop_reason.is_none() && (transaction_data[0].voided.is_none() || !transaction_data[0].voided.unwrap())  {
        if *verbose_mode {
            println!("Transaction on this connector is still active");
        }
        true
    }
    else if transaction_data[0].voided.is_some() && transaction_data[0].voided.unwrap() {
        if *verbose_mode {
            println!("Transaction on this connector was voided and hence is no longer active");
        }
        false
    }
    else {
        if *verbose_mode {
            println!("Transaction on this connector is not longer active");
        }
        false
    }
}
