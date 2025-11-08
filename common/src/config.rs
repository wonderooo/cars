use dotenvy::dotenv;
use serde::Deserialize;
use std::sync::LazyLock;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    dotenv().ok();

    let path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yaml".into());
    let config_file = std::fs::read_to_string(path).expect("failed to open config file");
    let mut unmarshalled =
        serde_yaml::from_str::<Config>(&config_file).expect("failed to parse config file");

    let copart_user = std::env::var("COPART_USER").expect("COPART_USER env var not set");
    let copart_password =
        std::env::var("COPART_PASSWORD").expect("COPART_PASSWORD env var not set");
    let data_bright_user =
        std::env::var("DATA_BRIGHT_USER").expect("DATA_BRIGHT_USER env var not set");
    let data_bright_password =
        std::env::var("DATA_BRIGHT_PASSWORD").expect("DATA_BRIGHT_PASSWORD env var not set");

    unmarshalled.copart.user = copart_user;
    unmarshalled.copart.password = copart_password;
    unmarshalled.data_bright.user = data_bright_user;
    unmarshalled.data_bright.password = data_bright_password;

    unmarshalled
});

#[derive(Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub copart: Copart,
    pub proxy: Proxy,
    pub minio: Minio,
    pub postgres: Postgres,
    pub kafka: Kafka,
    pub loki: Loki,
    pub data_bright: DataBright,
}

// Default impl for serde to skip copart field
#[derive(Default)]
pub struct Copart {
    pub user: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct Proxy {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct Minio {
    pub url: String,
    pub user: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct Postgres {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub db_name: String,
}

#[derive(Deserialize)]
pub struct Kafka {
    pub url: String,
}

#[derive(Deserialize)]
pub struct Loki {
    pub url: String,
}

#[derive(Deserialize)]
pub struct DataBright {
    pub host: String,
    pub port: u16,
    #[serde(skip)]
    pub user: String,
    #[serde(skip)]
    pub password: String,
    pub allow_domains: Vec<String>,
}
