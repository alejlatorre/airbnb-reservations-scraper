#[macro_use]
extern crate lazy_static;
mod config;
mod helpers;
mod models;
use config::globals::CONFIG;
use crossterm::event;
use helpers::engine::{get_summary, process_data};
use helpers::excel::{open_csv, open_xlsx};
use polars::prelude::*;
use std::collections::HashMap;
use std::io::{self, stdin, Write};

fn main() {
    let mut min_date = String::new();
    print!("Enter a search date: ");
    // If ends before search date or starts after search date, it won't be included
    io::stdout().flush().expect("Failed to flush stdout");
    stdin()
        .read_line(&mut min_date)
        .expect("Failed to read line");
    println!("{}", min_date);

    // let (output_filepath, data) = get_data(&min_date).expect("Failed to get data");
    // println!("Data has been written in {}", output_filepath);

    // let df: DataFrame = get_dataframe(data).expect("Failed to get dataframe");
    // println!("{:?}", df);

    // TODO: Delete in production
    let df: DataFrame = open_csv(CONFIG.example_csv.as_str()).expect("Failed to load csv");
    println!("{:?}", df.head(Some(5)));
    //

    let processed_df: DataFrame = process_data(df).expect("Failed to process data");
    println!("{:?}", processed_df.head(Some(5)));

    let summarized_df: DataFrame = get_summary(processed_df).expect("Failed to summarize data");
    println!("{:?}", summarized_df.head(Some(5)));

    // Auxiliar dataframes
    let mut hm_dataframes: HashMap<&str, DataFrame> = HashMap::new();
    let worksheets = vec![CONFIG.first_sheet.as_str(), CONFIG.second_sheet.as_str()];
    worksheets.iter().for_each(|sheet| {
        let excel_df: DataFrame =
            open_xlsx(CONFIG.base_xlsx.as_str(), sheet).expect("Failed to load xlsx");
        hm_dataframes.insert(sheet, excel_df);
    });

    let aux_df = hm_dataframes
        .get(CONFIG.first_sheet.as_str())
        .unwrap()
        .clone()
        .lazy()
        .join(
            hm_dataframes
                .get(CONFIG.second_sheet.as_str())
                .unwrap()
                .clone()
                .lazy(),
            [col("PROPIETARIO")],
            [col("PROPIETARIO")],
            JoinArgs::new(JoinType::Left),
        )
        .collect()
        .expect("Failed to join dataframes");

    // Consolidated df
    let cons_df = summarized_df
        .clone()
        .lazy()
        .join(
            aux_df.lazy(),
            [col("Listing")],
            [col("ANUNCIO")],
            JoinArgs::new(JoinType::Left),
        )
        .collect()
        .expect("Failed to join dataframes");

    println!("{:?}", cons_df.head(Some(5)));

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
