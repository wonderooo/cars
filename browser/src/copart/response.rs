/// Represents response for lot vehicles on the whole page
pub mod lot_search {
    use crate::impl_display_and_debug;
    use common::io::copart::LotVehicle;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::fmt::{Debug, Formatter};

    #[derive(Serialize, Deserialize, Clone, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct ApiResponse {
        pub return_code: i32,
        pub return_code_desc: String,
        pub data: Data,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct Data {
        pub query: Query,
        pub results: Results,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    pub struct Query {
        pub query: Vec<String>,
        pub filter: HashMap<String, serde_json::Value>,
        pub sort: Vec<String>,
        pub page: i32,
        pub size: i32,
        pub start: i32,
        pub watch_list_only: bool,
        pub free_form_search: bool,
        pub hide_images: bool,
        pub default_sort: bool,
        pub display_name: String,
        pub search_name: String,
        pub back_url: String,
        pub include_tag_by_field: HashMap<String, serde_json::Value>,
        pub raw_params: HashMap<String, serde_json::Value>,
        pub reload_watch_list_data: bool,
        pub specific_row_provided: bool,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct Results {
        pub total_elements: usize,
        pub content: Vec<Lot>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct Lot {
        pub drive_status: bool,
        pub dynamic_lot_details: DynamicLotDetails,
        pub vehicle_type_code: String,
        pub vehicle_cat_code: Option<String>,
        pub member_vehicle_type: String,
        pub odometer_uom: Option<String>,
        pub show_claim_form: bool,
        pub lot_plug_acv: f64,
        pub ready_for_replay_flag: bool,
        pub inspected_lot: bool,
        pub car_fax_report_available: bool,
        pub lot_number_str: String,
        pub lot_yard_same_as_kiosk_yard: bool,
        pub pwlot: bool,
        pub ln: i64,
        /// make
        pub mkn: String,
        pub lmg: String,
        pub lm: String,
        pub mmod: Option<String>,
        pub lcy: i32,
        pub fv: String,
        pub la: f64,
        pub rc: f64,
        pub obc: Option<String>,
        pub orr: f64,
        pub lfd: Option<Vec<serde_json::Value>>,
        pub ord: Option<String>,
        pub egn: Option<String>,
        pub cy: Option<String>,
        pub ld: String,
        pub yn: String,
        pub cuc: String,
        pub tz: String,
        pub lad: i64,
        pub at: String,
        pub aan: i32,
        pub hb: f64,
        pub ss: i32,
        pub bndc: String,
        pub bnp: f64,
        pub sbf: bool,
        pub ts: String,
        pub stt: String,
        pub td: String,
        pub tgc: String,
        pub tgd: String,
        pub dd: String,
        pub tims: String,
        pub lic: Vec<String>,
        pub gr: String,
        pub dtc: String,
        pub al: Option<String>,
        pub adt: String,
        pub ynumb: i32,
        pub phynumb: i32,
        pub bf: bool,
        pub cc: Option<String>,
        pub ymin: i32,
        pub off_flg: bool,
        pub loc_country: String,
        pub loc_state: String,
        pub htsmn: Option<String>,
        pub tmtp: Option<String>,
        pub myb: f64,
        pub lmc: String,
        pub lcc: Option<String>,
        pub sdd: Option<String>,
        pub bstl: Option<String>,
        pub lcd: Option<String>,
        pub clr: String,
        pub ft: Option<String>,
        pub hk: String,
        pub drv: Option<String>,
        pub ess: String,
        pub lsts: String,
        pub show_seller: bool,
        pub sstpflg: bool,
        pub hcr: bool,
        pub syn: String,
        pub ifs: bool,
        pub pbf: bool,
        pub crg: f64,
        pub brand: String,
        pub blucar: bool,
        pub hegn: Option<String>,
        pub lstg: i32,
        pub ldu: String,
        pub pcf: bool,
        pub btcf: bool,
        pub tpfs: bool,
        pub trf: bool,
        pub csc: String,
        pub mlf: bool,
        pub fcd: bool,
        pub slgc: String,
        pub slan: Option<String>,
        pub cfx: bool,
        pub hcfx: bool,
        pub hide_lane_item: bool,
        pub hide_grid_row: bool,
        pub is_pwlot: Option<bool>,
        pub lspa: f64,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
    #[serde(rename_all = "camelCase")]
    pub struct DynamicLotDetails {
        pub error_code: String,
        pub buyer_number: i32,
        pub source: String,
        pub buy_today_bid: f64,
        pub current_bid: i64,
        pub total_amount_due: f64,
        pub sealed_bid: bool,
        pub first_bid: bool,
        pub has_bid: bool,
        pub seller_reserve_met: bool,
        pub lot_sold: bool,
        pub bid_status: String,
        pub sale_status: String,
        pub counter_bid_status: String,
        pub starting_bid_flag: bool,
        pub buyer_high_bidder: bool,
        pub anonymous: bool,
        pub non_synced_buyer: bool,
    }

    impl_display_and_debug!(ApiResponse, |s: &ApiResponse, f: &mut Formatter<'_>| {
        writeln!(f, "LotSearch {{")?;
        for lot in &s.data.results.content {
            writeln!(
                f,
                "Lot {{ lot_number: {}, make: {}, model: {}, year: {} }}",
                lot.ln, lot.mkn, lot.lm, lot.lcy
            )?;
        }
        write!(f, "}}")
    });

    impl Into<Vec<LotVehicle>> for ApiResponse {
        fn into(self) -> Vec<LotVehicle> {
            self.data
                .results
                .content
                .into_iter()
                .map(|l| LotVehicle {
                    lot_number: l.ln as i32,
                    make: l.mkn,
                    year: l.lcy,
                })
                .collect()
        }
    }
}

pub mod lot_images {
    use common::io::copart::LotImages;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ApiResponse {
        pub return_code: i32,
        pub return_code_desc: String,
        pub data: Data,
    }

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Data {
        pub images_list: ImagesList,
    }

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ImagesList {
        pub total_elements: i32,
        pub content: Vec<Image>,
        pub facet_fields: Vec<String>,
        pub spell_check_list: Option<serde_json::Value>,
        pub suggestions: Option<serde_json::Value>,
        pub real_time: bool,
    }

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Image {
        pub swift_flag: bool,
        pub frame_count: i32,
        pub status: String,
        pub image_type_description: String,
        pub full_url: Option<String>,
        pub thumbnail_url: Option<String>,
        pub high_res_url: Option<String>,
        pub image_seq_number: i32,
        pub image_type_code: String,
        pub image_workflow_status: String,
        pub solr_high_res_url: Option<String>,
        pub solr_full_url: Option<String>,
        pub lot_number_str: String,
        pub image_type_enum: String,
        pub high_res: bool,
        pub ln: i64,
    }

    impl Into<Vec<LotImages>> for ApiResponse {
        fn into(self) -> Vec<LotImages> {
            self.data
                .images_list
                .content
                .into_iter()
                .map(|i| LotImages {
                    thumbnail_url: i.thumbnail_url,
                    full_url: i.full_url,
                    high_res_url: i.high_res_url,
                })
                .collect()
        }
    }
}
