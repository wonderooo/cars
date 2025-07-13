mod error;

use crate::copart::error::PersisterError;
use crate::orm::models::copart::NewLotVehicles;
use crate::orm::schema::lot_vehicle::dsl::lot_vehicle;
use crate::orm::schema::lot_vehicle::lot_number;
use crate::orm::PG_POOL;
use browser::browser::{
    CopartBrowserCmd, CopartBrowserCmdVariant, CopartBrowserResponse, CopartBrowserResponseVariant,
    LotSearchResponse,
};
use browser::pool::CopartBrowserPoolResponse;
use common::kafka::{AsyncRxFn, KafkaReceiver, KafkaSender};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
use uuid::Uuid;

pub struct CopartPersister;

impl CopartPersister {
    pub fn run(cancellation_token: CancellationToken) -> Arc<Notify> {
        info!("running copart persister");
        let sender = Arc::new(KafkaSender::new("localhost:9092"));
        let response_rx_done =
            KafkaReceiver::<CopartBrowserPoolResponse<CopartBrowserResponse>, AsyncRxFn>::new(
                "localhost:9092",
                "copart_response_lot_search_0",
                &["copart_response_lot_search"],
                Box::new(move |msg| {
                    let sender = Arc::clone(&sender);
                    Box::pin(async move {
                        if let Err(e) = Self::on_response(msg, sender).await {
                            error!("error on processing copart browser response: {e}")
                        }
                    })
                }),
                cancellation_token,
            )
            .run();

        let persister_done = Arc::new(Notify::new());
        tokio::spawn({
            let persister_done = persister_done.clone();
            async move {
                tokio::join!(response_rx_done.notified());
                info!("copart persister closed");
                persister_done.notify_waiters();
            }
        });

        persister_done
    }

    async fn on_response(
        msg: CopartBrowserPoolResponse<CopartBrowserResponse>,
        sender: impl AsRef<KafkaSender>,
    ) -> Result<(), PersisterError> {
        match msg.inner.variant {
            CopartBrowserResponseVariant::LotSearch(response) => {
                Self::on_lot_search_response(response?, sender).await?
            }
            CopartBrowserResponseVariant::LotDetails(response) => todo!(),
            CopartBrowserResponseVariant::LotImages(response) => todo!(),
        }

        Ok(())
    }

    async fn on_lot_search_response(
        lot_search_response: LotSearchResponse,
        sender: impl AsRef<KafkaSender>,
    ) -> Result<(), PersisterError> {
        let page_number = lot_search_response.page_number;
        let response = lot_search_response.response;

        debug!("received copart lot search response for page number `{page_number}`");
        let new_lot_vehicles: NewLotVehicles = response.into();
        debug!(
            "copart new lot vehicles to save `{}` for page number `{page_number}`",
            new_lot_vehicles.len()
        );

        let mut conn = PG_POOL.get().await?;
        let repeating_lns = lot_vehicle
            .select(lot_number)
            .filter(lot_number.eq_any(new_lot_vehicles.iter().map(|l| l.lot_number)))
            .load::<i32>(&mut conn)
            .await?;
        debug!(
            "repeating `{}` copart new lot vehicles for page number `{page_number}`",
            repeating_lns.len()
        );
        let unique_lot_vehicles = new_lot_vehicles
            .0
            .into_iter()
            .filter(|lv| !repeating_lns.contains(&lv.lot_number))
            .collect::<Vec<_>>();
        debug!(
            "unique `{}` copart new lot vehicles for page number `{page_number}`",
            unique_lot_vehicles.len()
        );
        let k = diesel::insert_into(crate::orm::schema::lot_vehicle::table)
            .values(&unique_lot_vehicles)
            .on_conflict_do_nothing() // Discard lot vehicles with already existing lot numbers
            .execute(&mut conn)
            .await?;
        debug!("inserted `{k}` copart new lot vehicles for page number `{page_number}`");

        // Send lot images cmd for all unique new lot vehicles concurrently
        futures::stream::iter(unique_lot_vehicles.iter())
            .for_each_concurrent(None, |lv| {
                let sender = sender.as_ref();
                async move {
                    let correlation_id = Uuid::new_v4();
                    sender
                        .send_with_key(
                            &CopartBrowserCmd {
                                correlation_id: correlation_id.as_simple().to_string(),
                                variant: CopartBrowserCmdVariant::LotImages(lv.lot_number),
                            },
                            correlation_id.as_simple().to_string(),
                            "copart_cmd_lot_images",
                        )
                        .await;
                }
            })
            .await;
        debug!(
            "sent `{}` copart lot images cmd for ln `{:?}` and page number `{page_number}`",
            unique_lot_vehicles.len(),
            unique_lot_vehicles
                .iter()
                .map(|lv| lv.lot_number)
                .collect::<Vec<_>>()
        );

        Ok(())
    }
}
