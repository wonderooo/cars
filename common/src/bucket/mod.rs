pub mod policies;

use crate::config::CONFIG;
use aws_config::environment::EnvironmentVariableCredentialsProvider;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::SharedCredentialsProvider;
use aws_sdk_s3::Client;
use std::sync::LazyLock;

pub fn init_s3() -> Client {
    let region = Region::new(CONFIG.s3.region.to_owned());
    let env_provider = EnvironmentVariableCredentialsProvider::new();
    let creds_provider = SharedCredentialsProvider::new(env_provider);

    let config = aws_config::SdkConfig::builder()
        .region(region)
        .credentials_provider(creds_provider)
        .behavior_version(BehaviorVersion::latest())
        .build();

    Client::new(&config)
}

pub static S3_CLIENT: LazyLock<Client> = LazyLock::new(|| init_s3());
