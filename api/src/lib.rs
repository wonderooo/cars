use utoipa::OpenApi;

pub mod domain;
pub mod error;
pub mod routes;

#[derive(OpenApi)]
#[openapi(paths(crate::routes::lot_vehicle::all, crate::routes::lot_vehicle::by_ln))]
pub struct Docs;
