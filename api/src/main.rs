use axum::routing::get;
use common::logging::setup_logging;
use common::persistence::init_pg_pool;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

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
        .route("/lot_vehicle", get(api::routes::lot_vehicle::all))
        .route(
            "/lot_vehicle/{lot_number}",
            get(api::routes::lot_vehicle::by_ln),
        )
        .with_state(pool)
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", api::Docs::openapi()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081")
        .await
        .expect("failed to bind");
    let app_done = serve(listener, app, cancellation_token.clone());

    info!("app started");
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
