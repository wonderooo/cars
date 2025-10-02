use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::Json;
use common::logging::setup_logging;
use common::persistence::models::copart::{LotImage, LotVehicle};
use common::persistence::schema::lot_vehicle::dsl::lot_vehicle;
use common::persistence::schema::lot_vehicle::lot_number;
use common::persistence::{init_pg_pool, PgPool};
use diesel::ExpressionMethods;
use diesel::{BelongingToDsl, GroupedBy, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use common::persistence::schema::lot_image::dsl::lot_image;
use common::persistence::schema::lot_image::{lot_vehicle_number, sequence_number};
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;
use tracing::info;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() {
    setup_logging("api");
    info!("starting app");
    let cancellation_token = CancellationToken::new();

    let pool = init_pg_pool();
    let app = axum::Router::new()
        // .route("/test", get(test))
        // .route("/test2/{lot_number}", get(test2))
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081")
        .await
        .expect("failed to bind");
    let app_done = serve(listener, app, cancellation_token.clone());

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl c event");
    info!("exiting");
    cancellation_token.cancel();
    app_done.notified().await;
    info!("exited");
}

fn serve(
    listener: tokio::net::TcpListener,
    app: axum::Router,
    cancellation_token: CancellationToken,
) -> Arc<Notify> {
    let done = Arc::new(Notify::new());

    tokio::spawn({
        let done = done.clone();
        async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    cancellation_token.cancelled().await;
                    info!("gracefully shutting down app");
                    done.notify_waiters();
                })
                .await
                .expect("failed to serve");
        }
    });

    done
}

// #[derive(Serialize)]
// struct BookWithPages {
//     #[serde(flatten)]
//     lot_vehicle: LotVehicle,
//     lot_images: Vec<LotImage>,
// }
//
// async fn test(
//     State(pool): State<PgPool>,
// ) -> Result<Json<Vec<BookWithPages>>, (StatusCode, String)> {
//     let mut conn = pool.get().await.unwrap();
//     let all_vehicles = lot_vehicle
//         .select(LotVehicle::as_select())
//         .limit(10)
//         .load(&mut conn)
//         .await
//         .unwrap();
//
//     let all_images = LotImage::belonging_to(&all_vehicles)
//         .select(LotImage::as_select())
//         .load(&mut conn)
//         .await
//         .unwrap();
//
//     let pages_per_book = all_images
//         .grouped_by(&all_vehicles)
//         .into_iter()
//         .zip(all_vehicles)
//         .map(|(pages, book)| BookWithPages {
//             lot_vehicle: book,
//             lot_images: pages,
//         })
//         .collect::<Vec<BookWithPages>>();
//
//     Ok(Json(pages_per_book))
// }
//
// async fn test2(
//     Path(ln): Path<i32>,
//     State(pool): State<PgPool>,
// ) -> Result<Json<BookWithPages>, (StatusCode, String)> {
//     let mut conn = pool.get().await.unwrap();
//     let all_vehicles = lot_vehicle
//         .filter(lot_number.eq(ln))
//         .select(LotVehicle::as_select())
//         .first(&mut conn)
//         .await
//         .optional()
//         .unwrap()
//         .unwrap();
//
//     let all_images = lot_image
//         .filter(lot_vehicle_number.eq(ln))
//         .order(sequence_number.asc())
//         .select(LotImage::as_select())
//         .load(&mut conn)
//         .await
//         .unwrap();
//
//     let bp = BookWithPages {
//         lot_vehicle: all_vehicles,
//         lot_images: all_images,
//     };
//
//     Ok(Json(bp))
// }
