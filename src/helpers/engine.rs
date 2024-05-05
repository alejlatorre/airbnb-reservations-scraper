use super::excel::write_to_excel_file;
use crate::config::globals::CONFIG;
use crate::models::reservation::{Reservation, ReservationTable};
use aws_credential_types::Credentials;
use aws_sdk_ssm::{config::Region, Client as ssm_client, Config};
use dotenv::dotenv;
use polars::prelude::*;
use regex::Regex;
use reqwest::blocking::Client as req_client;
use reqwest::header;
use reqwest::StatusCode;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::process;
use tokio::runtime::Runtime;

pub fn get_data(min_date: &str) -> Result<(String, Vec<Reservation>), reqwest::Error> {
    dotenv().ok();
    // Environment variables
    let ssm_prefix = env::var("SSM_PREFIX").expect("SSM_PREFIX is not set");

    // Hardcoded keys
    let access_key_id = "";
    let secret_access_key = "";

    // Import variables from AWS SSM
    let credentials = Credentials::from_keys(access_key_id, secret_access_key, None);
    let config = Config::builder()
        .credentials_provider(credentials)
        .region(Region::new("us-east-1"))
        .behavior_version_latest()
        .build();
    let client = ssm_client::from_conf(config);
    let response = Runtime::new().unwrap().block_on(async {
        client
            .get_parameters_by_path()
            .path(&ssm_prefix)
            .send()
            .await
    });

    let mut _params: HashMap<String, String> = HashMap::new();
    for param in response.unwrap().parameters.unwrap() {
        _params.insert(
            param.name.unwrap().to_string(),
            param.value.unwrap().to_string(),
        );
    }

    let _enable: String = _params
        .get(format!("{}/{}", ssm_prefix, "ENABLE").as_str())
        .unwrap()
        .to_string();
    if _enable != "true" {
        println!("The service is disabled");
        process::exit(1);
    }

    let _aat: String = _params
        .get(format!("{}/{}", ssm_prefix, "_AAT").as_str())
        .unwrap()
        .to_string();
    let airbnb_api_key: String = _params
        .get(format!("{}/{}", ssm_prefix, "AIRBNB_API_KEY").as_str())
        .unwrap()
        .to_string();

    // Cookies
    let cookies = format!("country=PE; _aat={}", _aat);

    // Headers
    let mut headers = header::HeaderMap::new();
    headers.insert("Accept", "*/*".parse().unwrap());
    headers.insert("Connection", "keep-alive".parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Host", "www.airbnb.com".parse().unwrap());
    headers.insert(
        "Referer",
        "https://www.airbnb.com/hosting/reservations"
            .parse()
            .unwrap(),
    );
    headers.insert("x-airbnb-api-key", airbnb_api_key.parse().unwrap());

    // Query parameters
    let mut query_params: HashMap<String, String> = HashMap::new();
    query_params.insert("locale".to_string(), "en".to_string());
    query_params.insert("currency".to_string(), "PEN".to_string());
    query_params.insert("_format".to_string(), "for_remy".to_string());
    query_params.insert("_limit".to_string(), "40".to_string());
    query_params.insert(
        "collection_strategy".to_string(),
        "for_reservations_list".to_string(),
    );
    query_params.insert("date_min".to_string(), min_date.to_string());
    query_params.insert("status".to_string(), "accepted,request".to_string());

    let mut _offset: i64 = 0;
    let delta: i64 = 40;
    let mut total_pages: i64 = 0;
    let mut page: i64 = 1;
    let mut reservations: Vec<Reservation> = Vec::new();

    let req_client = req_client::new();
    loop {
        let offset_value: String = _offset.to_string();
        query_params.insert("_offset".to_string(), offset_value);
        // println!("{:?}", query_params);

        let response = req_client
            .get("https://www.airbnb.com/api/v2/reservations")
            .headers(headers.clone())
            .query(&query_params)
            .header(reqwest::header::COOKIE, cookies.clone())
            .send()
            .expect("Failed to send request");

        match response.status() {
            StatusCode::OK => (),
            StatusCode::CREATED => (),
            StatusCode::ACCEPTED => (),
            StatusCode::UNAUTHORIZED => {
                println!("Status code: {}", response.status());
                // TODO: Update aat in cookies
                process::exit(1);
            }
            _ => {
                println!("Status code: {}", response.status());
                process::exit(1);
            }
        }

        let data: Value = response.json()?;
        if let Some(metadata) = data["metadata"].as_object() {
            if page == 1 {
                let total_records: i64 = metadata["total_count"].as_i64().unwrap_or(0);
                total_pages = metadata["page_count"].as_i64().unwrap_or(0);
                println!("Total records: {}", total_records);
                println!("Total pages: {}", total_pages);
            }
        }

        if let Some(records) = data["reservations"].as_array() {
            for record in records {
                let reservation = Reservation {
                    confirmation_code: record["confirmation_code"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    status: record["user_facing_status_localized"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    guest_user_full_name: record["guest_user"]["full_name"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    guest_user_phone: record["guest_user"]["phone"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    guest_details_number_of_adults: record["guest_details"]["number_of_adults"]
                        .as_f64()
                        .unwrap(),
                    guest_details_number_of_children: record["guest_details"]["number_of_children"]
                        .as_f64()
                        .unwrap(),
                    guest_details_number_of_infants: record["guest_details"]["number_of_infants"]
                        .as_f64()
                        .unwrap(),
                    start_date: record["start_date"].as_str().unwrap_or("").to_string(),
                    end_date: record["end_date"].as_str().unwrap_or("").to_string(),
                    nights: record["nights"].as_f64().unwrap(),
                    booked_date: record["booked_date"].as_str().unwrap_or("").to_string(),
                    listing_name: record["listing_name"].as_str().unwrap_or("").to_string(),
                    earnings: record["earnings"].as_str().unwrap_or("").to_string(),
                };
                reservations.push(reservation);
            }
        }

        println!("Page: {}", page);
        if page == total_pages {
            break;
        }
        page += 1;
        _offset += delta;
    }

    let filename: String = format!(
        "{}_{}.xlsx",
        CONFIG.output_file_prefix_1.as_str(),
        CONFIG.datetime_suffix.as_str()
    );
    write_to_excel_file(&filename, &reservations).expect("Failed to write to file");

    Ok((filename.to_string(), reservations))
}

fn row_to_column_structure(data: Vec<Reservation>) -> ReservationTable {
    let mut table = ReservationTable::default();
    for record in data {
        table.confirmation_code.push(record.confirmation_code);
        table.status.push(record.status);
        table.guest_user_full_name.push(record.guest_user_full_name);
        table.guest_user_phone.push(record.guest_user_phone);
        table
            .guest_details_number_of_adults
            .push(record.guest_details_number_of_adults);
        table
            .guest_details_number_of_children
            .push(record.guest_details_number_of_children);
        table
            .guest_details_number_of_infants
            .push(record.guest_details_number_of_infants);
        table.start_date.push(record.start_date);
        table.end_date.push(record.end_date);
        table.nights.push(record.nights);
        table.booked_date.push(record.booked_date);
        table.listing_name.push(record.listing_name);
        table.earnings.push(record.earnings);
    }
    return table;
}

fn column_to_series_structure(table: ReservationTable) -> Vec<Series> {
    let confirmation_code_series = Series::new("confirmation_code", table.confirmation_code);
    let status_series = Series::new("status", table.status);
    let guest_user_full_name_series =
        Series::new("guest_user_full_name", table.guest_user_full_name);
    let guest_user_phone_series = Series::new("guest_user_phone", table.guest_user_phone);
    let guest_details_number_of_adults_series = Series::new(
        "guest_details_number_of_adults",
        table.guest_details_number_of_adults,
    );
    let guest_details_number_of_children_series = Series::new(
        "guest_details_number_of_children",
        table.guest_details_number_of_children,
    );
    let guest_details_number_of_infants_series = Series::new(
        "guest_details_number_of_infants",
        table.guest_details_number_of_infants,
    );
    let start_date_series = Series::new("start_date", table.start_date);
    let end_date_series = Series::new("end_date", table.end_date);
    let nights_series = Series::new("nights", table.nights);
    let booked_date_series = Series::new("booked_date", table.booked_date);
    let listing_name_series = Series::new("listing_name", table.listing_name);
    let earnings_series = Series::new("earnings", table.earnings);

    vec![
        confirmation_code_series,
        status_series,
        guest_user_full_name_series,
        guest_user_phone_series,
        guest_details_number_of_adults_series,
        guest_details_number_of_children_series,
        guest_details_number_of_infants_series,
        start_date_series,
        end_date_series,
        nights_series,
        booked_date_series,
        listing_name_series,
        earnings_series,
    ]
}

pub fn get_dataframe(data: Vec<Reservation>) -> Result<DataFrame, PolarsError> {
    let table: ReservationTable = row_to_column_structure(data);
    let df = DataFrame::new(column_to_series_structure(table))?;
    Ok(df)
}

pub fn process_data(df: DataFrame) -> Result<DataFrame, PolarsError> {
    let pattern: Regex = Regex::new(r"([^\d,.]+)?([\d,]+\.\d+)").unwrap();

    let mut currencies: Vec<&str> = Vec::new();
    let mut amounts: Vec<f64> = Vec::new();

    for opt_s in df.column("earnings").into_iter() {
        let s = opt_s.str().unwrap();
        for s in s {
            let caps = pattern.captures(s.unwrap()).unwrap();
            currencies.push(caps.get(1).unwrap().as_str());
            amounts.push(
                caps.get(2)
                    .unwrap()
                    .as_str()
                    .replace(",", "")
                    .parse::<f64>()
                    .unwrap_or(0.0),
            );
        }
    }

    let currency_series = Series::new("currency", currencies);
    let amount_series = Series::new("amount", amounts);

    let _df = df
        .hstack(&[currency_series, amount_series])
        .expect("Failed to hstack");

    Ok(_df)
}

pub fn get_summary(df: DataFrame) -> Result<DataFrame, PolarsError> {
    let _df = df
        .lazy()
        .group_by([col("listing"), col("currency")])
        .agg([col("amount").sum().alias("total")])
        .sort(
            ["total"],
            SortMultipleOptions::new().with_order_descending(true),
        )
        .collect()?;
    Ok(_df)
}
