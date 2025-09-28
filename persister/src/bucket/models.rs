use common::io::copart::{Base64Blob, LotImageBlobsResponse};
use mime_guess::MimeGuess;

pub struct ImageInfo {
    pub blob: Base64Blob,
    pub url: String,
    pub mime_type: String,
    pub bucket_key: String,
}

pub struct NewLotImage {
    pub standard: Option<ImageInfo>,
    pub thumbnail: Option<ImageInfo>,
    pub high_res: Option<ImageInfo>,
    pub sequence_number: i32,
    pub image_type: String,
    pub lot_vehicle_number: i32,
}

pub struct NewLotImages(pub Vec<NewLotImage>);

impl From<LotImageBlobsResponse> for NewLotImages {
    fn from(value: LotImageBlobsResponse) -> Self {
        Self(
            value
                .response
                .0
                .into_iter()
                .map(|i| {
                    let image_info = |blob: Base64Blob, url: String, bucket_key: String| {
                        let mime = MimeGuess::from_path(&url).first_or_octet_stream();
                        ImageInfo {
                            blob,
                            url,
                            bucket_key,
                            mime_type: mime.to_string(),
                        }
                    };

                    let std = i.standard.map(|std| {
                        image_info(
                            std,
                            unsafe { i.standard_url.unwrap_unchecked() },
                            format!("{}_{}_standard", value.lot_number, i.sequence_number,),
                        )
                    });

                    let thumb = i.thumbnail.map(|thumb| {
                        image_info(
                            thumb,
                            unsafe { i.thumbnail_url.unwrap_unchecked() },
                            format!("{}_{}_thumbnail", value.lot_number, i.sequence_number,),
                        )
                    });

                    let high = i.high_res.map(|high| {
                        image_info(
                            high,
                            unsafe { i.high_res_url.unwrap_unchecked() },
                            format!("{}_{}_high-res", value.lot_number, i.sequence_number,),
                        )
                    });

                    NewLotImage {
                        standard: std,
                        thumbnail: thumb,
                        high_res: high,
                        sequence_number: i.sequence_number,
                        image_type: i.image_type,
                        lot_vehicle_number: value.lot_number,
                    }
                })
                .collect(),
        )
    }
}
