use crate::models::reservation::Reservation;
use calamine::{open_workbook_auto, Reader};
use polars::datatypes as dtype;
use polars::prelude::*;
use xlsxwriter::format::Format;
use xlsxwriter::{Workbook, Worksheet, XlsxError};

pub fn write_to_excel_file(
    filename: &str,
    reservations: &Vec<Reservation>,
) -> Result<(), XlsxError> {
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

pub fn write_to_excel_file_refac(filename: &str, data: DataFrame) -> Result<(), XlsxError> {
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
    worksheet.write_string(0, 13, "Currency", None)?;
    worksheet.write_string(0, 14, "Amount", None)?;
    worksheet.write_string(0, 15, "Owner", None)?;
    worksheet.write_string(0, 16, "Zone", None)?;
    worksheet.write_string(0, 17, "Comission (%)", None)?;
    worksheet.write_string(0, 18, "Comission (#)", None)?;

    let row_count = data.height() as u32;
    for i in 0..row_count {
        let row = data.get(i as usize).unwrap();
        row.iter().enumerate().for_each(|(col, cell)| {
            match cell.dtype() {
                dtype::DataType::String => worksheet.write_string(
                    i + 1,
                    col as u16,
                    cell.to_string().trim_matches('"'),
                    None,
                ),
                dtype::DataType::Int64 => worksheet.write_number(
                    i + 1,
                    col as u16,
                    cell.to_string().parse::<f64>().unwrap(),
                    None,
                ),
                dtype::DataType::Float64 => worksheet.write_number(
                    i + 1,
                    col as u16,
                    cell.to_string().parse::<f64>().unwrap(),
                    None,
                ),
                _ => worksheet.write_string(
                    i + 1,
                    col as u16,
                    cell.to_string().trim_matches('"'),
                    None,
                ),
            }
            .expect("Failed to write cell")
        });
    }
    workbook.close()
}

pub fn open_csv(filename: &str) -> Result<DataFrame, PolarsError> {
    let df = CsvReader::from_path(filename)?.has_header(true).finish()?;
    Ok(df)
}

pub fn open_xlsx(filename: &str, sheet_name: &str) -> Result<DataFrame, PolarsError> {
    // Open workbook and define sheet
    let mut workbook = open_workbook_auto(filename).expect("Failed to open workbook");
    let worksheet = workbook
        .worksheet_range(sheet_name)
        .expect("Failed to open worksheet");

    // Schema
    let mut headers = Vec::new();
    let mut records = Vec::new();

    // Get rows and columns
    for (i, row) in worksheet.rows().enumerate() {
        if i == 0 {
            headers = row.iter().map(|column| column.to_string()).collect();
        } else {
            let record: Vec<_> = row
                .iter()
                .map(|cell| match cell {
                    calamine::Data::String(s) => s.clone(),
                    calamine::Data::Float(f) => f.to_string(),
                    calamine::Data::DateTime(f) => f.to_string(),
                    calamine::Data::Int(i) => i.to_string(),
                    calamine::Data::Bool(b) => b.to_string(),
                    calamine::Data::Error(e) => e.to_string(),
                    calamine::Data::Empty => "".to_string(),
                    _ => cell.to_string(),
                })
                .collect();
            records.push(record);
        }
    }

    let mut series = Vec::with_capacity(headers.len());
    for (i, header) in headers.iter().enumerate() {
        let data: Vec<String> = records.iter().map(|record| record[i].clone()).collect();
        let s: Series = Series::new(header, data);
        series.push(s);
    }

    let _df = DataFrame::new(series).expect("Failed to create dataframe");

    Ok(_df)
}
