use dotenv::dotenv;
use std::env;

pub struct Config {
    pub example_csv: String,
    pub base_xlsx: String,
    pub first_sheet: String,
    pub second_sheet: String,
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();

        let example_csv = env::var("EXAMPLE_CSV").expect("EXAMPLE_CSV must be set");
        let base_xlsx = env::var("BASE_XLSX").expect("BASE_XLSX must be set");
        let first_sheet = env::var("FIRST_SHEET").expect("FIRST_SHEET is not set");
        let second_sheet = env::var("SECOND_SHEET").expect("SECOND_SHEET must be set");

        Config {
            example_csv,
            base_xlsx,
            first_sheet,
            second_sheet,
        }
    }
}

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}
