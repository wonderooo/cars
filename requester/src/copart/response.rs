use crate::copart::CopartImageSet;

pub struct CopartRequesterPoolResponse {
    pub inner: CopartRequesterResponse,
    pub n_worker: usize,
}

pub enum CopartRequesterResponse {
    LotImagesBlob(LotImagesBlobResponse),
}

pub struct LotImagesBlobResponse {
    pub images: Vec<CopartImageSet>,
}
