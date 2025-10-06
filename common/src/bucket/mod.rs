pub mod models;
pub mod policies;

use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::Client;
use std::sync::LazyLock;

pub fn init_minio() -> Client {
    let base_url: BaseUrl = "http://localhost:9000".parse().expect("invalid url");
    let provider = StaticProvider::new("root", "secret123", None);
    Client::new(base_url, Some(Box::new(provider)), None, None).expect("failed to create client")
}

pub static MINIO_CLIENT: LazyLock<Client> = LazyLock::new(|| init_minio());
