#[macro_use]
extern crate lazy_static;
mod config;
mod helpers;
mod models;
use config::globals::CONFIG;
use crossterm::event;
use helpers::engine::{get_data, get_dataframe, process_data};
use helpers::excel::{open_xlsx, write_to_excel_file_refac};
use polars::prelude::*;
use std::collections::HashMap;
use std::env;
use std::io::{self, stdin, Write};
use std::path::PathBuf;

fn get_path(filename: &str) -> String {
    let cwd = env::current_dir().expect("Failed to get current directory");
    let _filename = PathBuf::from(cwd.join(filename))
        .as_path()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();
    _filename
}

fn main() {
    if cfg!(debug_assertions) {
        println!("Running in debug mode");
    } else {
        println!("Running in release mode");
        if let Ok(exe_path) = env::current_exe() {
            if let Some(dir_path) = exe_path.parent() {
                env::set_current_dir(dir_path).expect("Failed to set current directory");
            }
        }
    }

    let mut min_date = String::new();
    print!("Enter a search date: ");
    io::stdout().flush().expect("Failed to flush stdout");
    stdin()
        .read_line(&mut min_date)
        .expect("Failed to read line");
    println!("{}", min_date);

    println!("Starting extraction step...");
    let (output_filepath, data) = get_data(&min_date).expect("Failed to get data");
    println!("Data has been extracted and written in {}", output_filepath);

    println!("Starting processing step...");
    let df: DataFrame = get_dataframe(data).expect("Failed to get dataframe");
    let processed_df: DataFrame = process_data(df).expect("Failed to process data");

    // Auxiliar dataframes
    let mut hm_dataframes: HashMap<&str, DataFrame> = HashMap::new();
    let worksheets = vec![
        CONFIG.first_sheet.as_str(),
        CONFIG.second_sheet.as_str(),
        CONFIG.third_sheet.as_str(),
    ];
    worksheets.iter().for_each(|sheet| {
        let excel_df: DataFrame =
            open_xlsx(&get_path(CONFIG.base_xlsx.as_str()), sheet).expect("Failed to load xlsx");
        hm_dataframes.insert(sheet, excel_df);
    });

    let base_comission = hm_dataframes
        .get(CONFIG.third_sheet.as_str())
        .unwrap()
        .get(0)
        .unwrap()
        .get(0)
        .unwrap()
        .to_string()
        .trim_matches('"')
        .parse::<f64>()
        .expect("Failed to parse f64");

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
    let cons_df = processed_df
        .clone()
        .lazy()
        .join(
            aux_df.lazy(),
            [col("listing_name")],
            [col("ANUNCIO")],
            JoinArgs::new(JoinType::Left),
        )
        .with_column(
            col("Comision")
                .cast(DataType::Float64)
                .fill_null(base_comission)
                .alias("Comision"),
        )
        .with_column((col("amount") * col("Comision")).alias("commission_earnings"))
        .collect()
        .expect("Failed to join dataframes");

    let _output_filepath = get_path(CONFIG.output_file.as_str());
    write_to_excel_file_refac(&_output_filepath, cons_df.clone())
        .expect("Failed to generate Excel");

    println!("Data has been processed...");
    println!("The Excel file was generated in {}", _output_filepath);
    println!("Showing the first 5 rows...");
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
