use dotenv::dotenv;
use std::env;

pub struct Config {
    pub example_csv: String,
    pub base_xlsx: String,
    pub first_sheet: String,
    pub second_sheet: String,
    pub third_sheet: String,
    pub output_file: String,
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();

        let example_csv: String = env::var("EXAMPLE_CSV").expect("EXAMPLE_CSV must be set");
        let base_xlsx: String = env::var("BASE_XLSX").expect("BASE_XLSX must be set");
        let first_sheet: String = env::var("FIRST_SHEET").expect("FIRST_SHEET must be set");
        let second_sheet: String = env::var("SECOND_SHEET").expect("SECOND_SHEET must be set");
        let third_sheet: String = env::var("THIRD_SHEET").expect("THIRD_SHEET must be set");
        let output_file: String = env::var("OUTPUT_FILE").expect("OUTPUT_FILE must be set");

        Config {
            example_csv,
            base_xlsx,
            first_sheet,
            second_sheet,
            third_sheet,
            output_file,
        }
    }
}

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}
