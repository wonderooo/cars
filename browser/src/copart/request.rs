pub mod lot_search {
    use common::io::copart::{DateTimeRfc3339, LotYear, PageNumber};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SearchRequest {
        pub query: Vec<String>,
        pub filter: HashMap<String, serde_json::Value>,
        pub sort: Vec<String>,
        pub page: usize,
        pub size: usize,
        pub start: usize,
        pub watch_list_only: bool,
        pub free_form_search: bool,
        pub hide_images: bool,
        pub default_sort: bool,
        pub specific_row_provided: bool,
        pub display_name: String,
        pub search_name: String,
        pub back_url: String,
        pub include_tag_by_field: HashMap<String, serde_json::Value>,
        pub raw_params: HashMap<String, serde_json::Value>,
    }

    impl SearchRequest {
        pub fn new(page: PageNumber) -> Self {
            let size = 1000;
            Self {
                query: vec!["*".to_string()],
                filter: HashMap::new(),
                sort: vec![
                    "salelight_priority asc".to_string(),
                    "member_damage_group_priority asc".to_string(),
                    "auction_date_type desc".to_string(),
                    "auction_date_utc asc".to_string(),
                ],
                page,
                size,
                start: page * size,
                watch_list_only: false,
                free_form_search: true,
                hide_images: false,
                default_sort: false,
                specific_row_provided: false,
                display_name: String::new(),
                search_name: String::new(),
                back_url: String::new(),
                include_tag_by_field: HashMap::new(),
                raw_params: HashMap::new(),
            }
        }

        pub fn with_auction_date(
            mut self,
            date_start: &DateTimeRfc3339,
            date_end: &DateTimeRfc3339,
        ) -> Self {
            self.filter.insert(
                "SDAT".to_string(),
                [format!(
                    "auction_date_utc:[\"{date_start}\" TO \"{date_end}\"]"
                )]
                .into(),
            );
            self
        }

        pub fn with_year(mut self, year_start: &LotYear, year_end: &LotYear) -> Self {
            self.filter.insert(
                "YEAR".to_string(),
                [format!("lot_year:[{year_start} TO {year_end}]")].into(),
            );
            self
        }
    }
}

pub mod login {
    use common::config::CONFIG;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LoginRequest {
        pub username: String,
        pub account_type: String,
        pub password: String,
        pub account_type_value: String,
        pub login_location_info: LoginLocationInfo,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LoginLocationInfo {
        pub country_code: String,
        pub country_name: String,
        pub state_name: String,
        pub state_code: String,
        pub city_name: String,
        pub latitude: f64,
        pub longitude: f64,
        pub zip_code: String,
        pub time_zone: String,
    }

    impl LoginRequest {
        pub fn new() -> Self {
            Self {
                username: CONFIG.copart.user.to_owned(),
                account_type: "0".into(),
                password: CONFIG.copart.password.to_owned(),
                account_type_value: "0".into(),
                login_location_info: LoginLocationInfo {
                    country_code: "POL".into(),
                    country_name: "Poland".into(),
                    state_name: "Mazowieckie".into(),
                    state_code: "".into(),
                    city_name: "Warsaw".into(),
                    latitude: 52.22977,
                    longitude: 21.01178,
                    zip_code: "05-077".into(),
                    time_zone: "+02:00".into(),
                },
            }
        }
    }
}
