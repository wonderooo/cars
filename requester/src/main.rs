use browser::browser::{CopartBrowserResponse, CopartBrowserResponseVariant};
use browser::pool::CopartBrowserPoolResponse;
use common::kafka::{AsyncRxFn, KafkaReceiver, KafkaSender};
use common::logging::setup_logging;
use requester::copart::CopartRequester;
use std::sync::Arc;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

#[tokio::main]
async fn main() {
    setup_logging("requester");
    let cancellation_token = CancellationToken::new();

    let copart_requester = CopartRequester::new();
    let sender = Arc::new(KafkaSender::new("localhost:9092"));
    let rx_done =
        KafkaReceiver::<CopartBrowserPoolResponse<CopartBrowserResponse>, AsyncRxFn>::new(
            "localhost:9092",
            "copart_response_lot_images_0",
            &["copart_response_lot_images"],
            Box::new(move |general_response| {
                let copart_requester = CopartRequester::clone(&copart_requester);
                let sender = Arc::clone(&sender);
                Box::pin(async move {
                    match general_response.inner.variant {
                        CopartBrowserResponseVariant::LotImages(Err(e)) => {
                            error!("error processing copart lot images response: {e}")
                        }
                        CopartBrowserResponseVariant::LotImages(Ok(lot_images)) => {
                            debug!(
                                "copart lot images response for ln `{}`",
                                lot_images.lot_number
                            );

                            let start = Instant::now();
                            let images =
                                copart_requester.download_images(lot_images.response).await;
                            debug!(
                                "copart lot images download took `{}ms` for ln `{}`",
                                start.elapsed().as_millis(),
                                lot_images.lot_number
                            );

                            sender
                                .send(&images, "copart_response_blob_lot_images")
                                .await;
                        }
                        _ => todo!(),
                    }
                })
            }),
            cancellation_token.clone(),
        )
        .run();

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl c event");
    info!("exiting");
    cancellation_token.cancel();
    rx_done.notified().await;
    info!("exited");
}
