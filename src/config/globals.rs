use chrono::Local;
use dotenv::dotenv;
use std::env;

pub struct Config {
    pub example_csv: String,
    pub base_xlsx: String,
    pub first_sheet: String,
    pub second_sheet: String,
    pub third_sheet: String,
    pub output_file_prefix_1: String,
    pub output_file_prefix_2: String,
    pub datetime_suffix: String,
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();

        let example_csv: String = env::var("EXAMPLE_CSV").expect("EXAMPLE_CSV must be set");
        let base_xlsx: String = env::var("BASE_XLSX").expect("BASE_XLSX must be set");
        let first_sheet: String = env::var("FIRST_SHEET").expect("FIRST_SHEET must be set");
        let second_sheet: String = env::var("SECOND_SHEET").expect("SECOND_SHEET must be set");
        let third_sheet: String = env::var("THIRD_SHEET").expect("THIRD_SHEET must be set");
        let output_file_prefix_1: String =
            env::var("OUTPUT_FILE_PREFIX_1").expect("OUTPUT_FILE_PREFIX_1 must be set");
        let output_file_prefix_2: String =
            env::var("OUTPUT_FILE_PREFIX_2").expect("OUTPUT_FILE_PREFIX_2 must be set");
        let datetime_suffix: String = Local::now().format("%Y%m%d%H%M%S").to_string();

        Config {
            example_csv,
            base_xlsx,
            first_sheet,
            second_sheet,
            third_sheet,
            output_file_prefix_1,
            output_file_prefix_2,
            datetime_suffix,
        }
    }
}

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}
