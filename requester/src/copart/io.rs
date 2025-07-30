use browser::response::lot_images;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum CopartRequesterResponse {
    LotImageBlobs { images: Vec<CopartImageSet> },
}

pub type Base64Blob = String;
#[derive(Debug, Serialize, Deserialize)]
pub struct CopartImageSet {
    pub standard: Option<Base64Blob>,
    pub high_res: Option<Base64Blob>,
    pub thumbnail: Option<Base64Blob>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CopartRequesterCmd {
    LotImageBlobs { cmds: Vec<CopartImageBlobCmd> },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CopartImageBlobCmd {
    pub thumbnail_url: Option<String>,
    pub full_url: Option<String>,
    pub high_res_url: Option<String>,
}

impl From<lot_images::ApiResponse> for CopartRequesterCmd {
    fn from(value: lot_images::ApiResponse) -> Self {
        Self::LotImageBlobs {
            cmds: value
                .data
                .images_list
                .content
                .into_iter()
                .map(|image| CopartImageBlobCmd {
                    thumbnail_url: image.thumbnail_url,
                    full_url: image.full_url,
                    high_res_url: image.high_res_url,
                })
                .collect(),
        }
    }
}
