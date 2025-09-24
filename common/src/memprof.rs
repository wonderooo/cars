use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use std::sync::Arc;
use tokio::net::ToSocketAddrs;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub struct MemProf;

impl MemProf {
    pub fn start(
        addr: impl ToSocketAddrs + Send + 'static,
        cancellation_token: CancellationToken,
    ) -> Arc<Notify> {
        let done = Arc::new(Notify::new());
        tokio::spawn({
            let done = done.clone();
            async move {
                let app = axum::Router::new().route(
                    "/memprof/flamegraph",
                    axum::routing::get(Self::handle_get_heap_flamegraph),
                );
                let listener = tokio::net::TcpListener::bind(addr)
                    .await
                    .expect("failed to bind");

                axum::serve(listener, app)
                    .with_graceful_shutdown(async move {
                        cancellation_token.cancelled().await;
                        done.notify_waiters();
                    })
                    .await
                    .expect("failed to serve");
            }
        });

        done
    }

    pub async fn handle_get_heap_flamegraph() -> Result<impl IntoResponse, (StatusCode, String)> {
        let mut prof_ctl = jemalloc_pprof::PROF_CTL.as_ref().unwrap().lock().await;
        Self::require_profiling_activated(&prof_ctl)?;
        let svg = prof_ctl
            .dump_flamegraph()
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
        Response::builder()
            .header(CONTENT_TYPE, "image/svg+xml")
            .body(Body::from(svg))
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
    }

    fn require_profiling_activated(
        prof_ctl: &jemalloc_pprof::JemallocProfCtl,
    ) -> Result<(), (StatusCode, String)> {
        if prof_ctl.activated() {
            Ok(())
        } else {
            Err((StatusCode::FORBIDDEN, "heap profiling not activated".into()))
        }
    }
}
