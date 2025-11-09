use aws_sdk_s3::types::{BucketLocationConstraint, CreateBucketConfiguration};
use common::bucket::S3_CLIENT;
use common::retry_async;
use std::time::Duration;

const BUCKET_NAME: &str = "cars-lot-images";

#[tokio::main]
async fn main() {
    println!("Creating absent bucket");
    let bucket_list_response = retry_async(Duration::from_millis(200), 5, || {
        S3_CLIENT.list_buckets().send()
    })
    .await
    .expect("failed to check bucket");

    let bucket_names = bucket_list_response
        .buckets()
        .iter()
        .map(|b| b.name.clone().unwrap_or("".to_string()))
        .collect::<Vec<_>>();

    if bucket_names.contains(&BUCKET_NAME.to_owned()) {
        println!("Bucket already exists");
        return;
    }

    retry_async(Duration::from_millis(200), 5, || {
        S3_CLIENT
            .create_bucket()
            .bucket(BUCKET_NAME)
            .create_bucket_configuration(
                CreateBucketConfiguration::builder()
                    .location_constraint(BucketLocationConstraint::EuCentral1)
                    .build(),
            )
            .send()
    })
    .await
    .expect("failed to set bucket policy");
    println!("Bucket created");
}
