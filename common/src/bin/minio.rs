use common::bucket::policies::public_bucket_policy;
use common::bucket::MINIO_CLIENT;
use common::retry_async;
use minio::s3::types::S3Api;
use std::time::Duration;

const BUCKET_NAME: &str = "lot-images";

#[tokio::main]
async fn main() {
    let bucket_exist_response = retry_async(Duration::from_millis(200), 5, || {
        MINIO_CLIENT.bucket_exists(BUCKET_NAME).send()
    })
    .await
    .expect("failed to check bucket");

    if !bucket_exist_response.exists {
        println!("bucket {} does not exist", BUCKET_NAME);
        retry_async(Duration::from_millis(200), 5, || {
            MINIO_CLIENT.create_bucket(BUCKET_NAME).send()
        })
        .await
        .expect("failed to create bucket");
        println!("bucket {} created", BUCKET_NAME);
    } else {
        println!("bucket {} exists", BUCKET_NAME);
    }

    retry_async(Duration::from_millis(200), 5, || {
        MINIO_CLIENT
            .put_bucket_policy(BUCKET_NAME)
            .config(public_bucket_policy(BUCKET_NAME))
            .send()
    })
    .await
    .expect("failed to set bucket policy");
    println!("bucket {} policy set", BUCKET_NAME);
}
