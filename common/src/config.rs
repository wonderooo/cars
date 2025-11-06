use serde::Deserialize;
use std::sync::LazyLock;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yaml".into());
    let config_file = std::fs::read_to_string(path).expect("failed to open config file");
    serde_yaml::from_str(&config_file).expect("failed to parse config file")
});

#[derive(Deserialize)]
pub struct Config {
    pub copart: Copart,
    pub proxy: Proxy,
    pub minio: Minio,
    pub postgres: Postgres,
    pub kafka: Kafka,
    pub loki: Loki,
    pub data_bright: DataBright,
}

#[derive(Deserialize)]
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
    pub user: String,
    pub password: String,
    pub allow_domains: Vec<String>,
}
