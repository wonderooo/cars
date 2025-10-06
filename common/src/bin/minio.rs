use common::bucket::policies::public_bucket_policy;
use common::bucket::MINIO_CLIENT;
use minio::s3::types::S3Api;

const BUCKET_NAME: &str = "lot-images";

#[tokio::main]
async fn main() {
    let bucket_exist_response = MINIO_CLIENT
        .bucket_exists(BUCKET_NAME)
        .send()
        .await
        .expect("failed to check bucket");

    if bucket_exist_response.exists {
        println!("bucket {} exists", BUCKET_NAME);
        MINIO_CLIENT
            .delete_and_purge_bucket(BUCKET_NAME)
            .await
            .expect("failed to delete bucket");
        println!("bucket {} deleted", BUCKET_NAME);
    } else {
        println!("bucket {} does not exist", BUCKET_NAME);
    }

    MINIO_CLIENT
        .create_bucket(BUCKET_NAME)
        .send()
        .await
        .expect("failed to create bucket");
    println!("bucket {} created", BUCKET_NAME);

    MINIO_CLIENT
        .put_bucket_policy(BUCKET_NAME)
        .config(public_bucket_policy(BUCKET_NAME))
        .send()
        .await
        .expect("failed to set bucket policy");
    println!("bucket {} policy set", BUCKET_NAME);
}
