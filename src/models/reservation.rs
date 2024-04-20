pub struct Reservation {
    pub confirmation_code: String,
    pub status: String,
    pub guest_user_full_name: String,
    pub guest_user_phone: String,
    pub guest_details_number_of_adults: f64,
    pub guest_details_number_of_children: f64,
    pub guest_details_number_of_infants: f64,
    pub start_date: String,
    pub end_date: String,
    pub nights: f64,
    pub booked_date: String,
    pub listing_name: String,
    pub earnings: String,
}
