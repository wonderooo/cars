pub mod lot_search {
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
        pub fn new(page: usize) -> Self {
            let size = 100;
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
    }
}
