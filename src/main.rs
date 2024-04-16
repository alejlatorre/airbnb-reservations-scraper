use chrono::{DateTime, Local};
use crossterm::event;
use dotenv::dotenv;
use reqwest::blocking::Client;
use reqwest::header;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::io::{self, stdin};
use xlsxwriter::format::Format;
use xlsxwriter::{Workbook, Worksheet, XlsxError};

struct Reservation {
    confirmation_code: String,
    status: String,
    guest_user_full_name: String,
    guest_user_phone: String,
    guest_details_number_of_adults: f64,
    guest_details_number_of_children: f64,
    guest_details_number_of_infants: f64,
    start_date: String,
    end_date: String,
    nights: f64,
    booked_date: String,
    listing_name: String,
    earnings: String,
}

fn write_to_excel_file(filename: &str, reservations: &Vec<Reservation>) -> Result<(), XlsxError> {
    let workbook: Workbook = Workbook::new(filename)?;
    let mut worksheet: Worksheet = workbook.add_worksheet(None)?;

    // Write headers
    worksheet.write_string(0, 0, "Confirmation code", None)?;
    worksheet.write_string(0, 1, "Status", None)?;
    worksheet.write_string(0, 2, "Guest name", None)?;
    worksheet.write_string(0, 3, "Contact", None)?;
    worksheet.write_string(0, 4, "# of adults", None)?;
    worksheet.write_string(0, 5, "# of children", None)?;
    worksheet.write_string(0, 6, "# of infants", None)?;
    worksheet.write_string(0, 7, "Start date", None)?;
    worksheet.write_string(0, 8, "End date", None)?;
    worksheet.write_string(0, 9, "# of nights", None)?;
    worksheet.write_string(0, 10, "Booked", None)?;
    worksheet.write_string(0, 11, "Listing", None)?;
    worksheet.write_string(0, 12, "Earnings", None)?;

    // Iterate over the reservations and write data
    for (index, reservation) in reservations.iter().enumerate() {
        let row = (index + 1) as u32; // Start writing from the second row
        worksheet.write_string(row, 0, &reservation.confirmation_code, None)?;
        worksheet.write_string(row, 1, &reservation.status, None)?;
        worksheet.write_string(row, 2, &reservation.guest_user_full_name, None)?;
        worksheet.write_string(row, 3, &reservation.guest_user_phone, None)?;
        worksheet.write_number(
            row,
            4,
            reservation.guest_details_number_of_adults,
            Some(&Format::new().set_num_format("#,##0")),
        )?;
        worksheet.write_number(
            row,
            5,
            reservation.guest_details_number_of_children,
            Some(&Format::new().set_num_format("#,##0")),
        )?;
        worksheet.write_number(
            row,
            6,
            reservation.guest_details_number_of_infants,
            Some(&Format::new().set_num_format("#,##0")),
        )?;
        worksheet.write_string(row, 7, &reservation.start_date, None)?;
        worksheet.write_string(row, 8, &reservation.end_date, None)?;
        worksheet.write_number(
            row,
            9,
            reservation.nights,
            Some(&Format::new().set_num_format("#,##0")),
        )?;
        worksheet.write_string(row, 10, &reservation.booked_date, None)?;
        worksheet.write_string(row, 11, &reservation.listing_name, None)?;
        worksheet.write_string(row, 12, &reservation.earnings, None)?;
    }

    workbook.close()
}

fn get_data(min_date: &str) -> Result<String, reqwest::Error> {
    dotenv().ok();
    let client = Client::new();

    // Environment variables
    let _aat = env::var("_AAT").expect("AAT is not set");
    let airbnb_api_key = env::var("AIRBNB_API_KEY").expect("AIRBNB_API_KEY is not set");

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

    loop {
        let offset_value: String = _offset.to_string();
        query_params.insert("_offset".to_string(), offset_value);
        // println!("{:?}", query_params);

        let response = client
            .get("https://www.airbnb.com/api/v2/reservations")
            .headers(headers.clone())
            .query(&query_params)
            .header(reqwest::header::COOKIE, cookies.clone())
            .send()
            .expect("Failed to send request");

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

    let now: DateTime<Local> = Local::now();
    let filename: String = format!("reservations_{}.xlsx", now.format("%Y%m%d%H%M%S"));
    write_to_excel_file(&filename, &reservations).expect("Failed to write to file");

    Ok(filename.to_string())
}

fn main() {
    let mut min_date = String::new();
    print!("Enter a search date: ");
    // If ends before search date or starts after search date, it won't be included
    io::stdout().flush().expect("Failed to flush stdout");
    stdin()
        .read_line(&mut min_date)
        .expect("Failed to read line");
    println!("{}", min_date);

    let output_filepath: String = get_data(&min_date).expect("Failed to get data");
    println!("Data has been written in {}", output_filepath);
    println!("Press any key to close...");
    io::stdout().flush().expect("Failed to flush stdout");

    crossterm::terminal::enable_raw_mode().expect("Failed to enable raw mode");
    loop {
        if event::poll(std::time::Duration::from_secs(0)).expect("Failed to poll event") {
            if let event::Event::Key(_) = event::read().expect("Failed to read event") {
                break;
            }
        }
    }
    crossterm::terminal::disable_raw_mode().expect("Failed to disable raw mode");
}