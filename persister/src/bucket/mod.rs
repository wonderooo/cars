pub mod models;

use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::{Client, Config};
use std::sync::LazyLock;

pub static MINIO_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let credentials = Credentials::new("root", "secret123", None, None, "minio");
    let config = Config::builder()
        .region(Some(Region::new("us-east-1")))
        .endpoint_url("http://localhost:9000")
        .credentials_provider(credentials)
        .force_path_style(true)
        .build();
    Client::from_conf(config)
});

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_s3::primitives::ByteStream;

    #[tokio::test]
    async fn test_minio_client() {
        MINIO_CLIENT
            .put_object()
            .bucket("lot-images")
            .key("key")
            .body(ByteStream::from("body".to_string().into_bytes()))
            .send()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_minio_client_2() -> Result<(), Box<dyn std::error::Error>> {
        let bucket_name = "lot-images";

        let public_policy = format!(
            r#"{{
            "Version":"2012-10-17",
            "Statement":[{{
                "Sid":"PublicReadGetObject",
                "Effect":"Allow",
                "Principal":"*",
                "Action":["s3:GetObject"],
                "Resource":["arn:aws:s3:::{bucket}/*"]
            }}]
        }}"#,
            bucket = bucket_name
        );

        match MINIO_CLIENT
            .put_bucket_policy()
            .bucket(bucket_name)
            .policy(public_policy)
            .send()
            .await
        {
            Ok(_) => println!("Bucket {} is now public!", bucket_name),
            Err(e) => println!("Failed to set policy: {:?}", e),
        }

        Ok(())
    }
}
