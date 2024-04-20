use crate::models::reservation::Reservation;

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
