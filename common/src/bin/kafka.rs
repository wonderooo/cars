use common::config::CONFIG;
use common::kafka::KafkaAdmin;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let admin = KafkaAdmin::new(CONFIG.kafka.url.to_owned());
    admin
        .create_absent_topic("copart_cmd_lot_search")
        .await
        .expect("failed to recreate `copart_cmd_lot_search` topic");
    admin
        .create_absent_topic("copart_cmd_lot_search")
        .await
        .expect("failed to recreate `copart_cmd_lot_search` topic");
    admin
        .create_absent_topic("copart_cmd_lot_images")
        .await
        .expect("failed to recreate `copart_cmd_lot_images` topic");
    admin
        .create_absent_topic("copart_response_lot_search")
        .await
        .expect("failed to recreate `copart_response_lot_search` topic");
    admin
        .create_absent_topic_with_opts(
            "copart_response_lot_image_blobs",
            &HashMap::from([
                ("max.message.bytes", "100000000"),
                ("retention.ms", "1800000"),
            ]),
        )
        .await
        .expect("failed to recreate `copart_response_lot_image_blobs` topic");
    admin
        .create_absent_topic("copart_response_lot_images")
        .await
        .expect("failed to recreate `copart_response_lot_images` topic");
}
