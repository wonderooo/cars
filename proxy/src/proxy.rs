use base64::Engine;
use bytes::Bytes;
use common::config::CONFIG;
use http::StatusCode;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper::{Method, Request, Response};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info};

type ServerBuilder = hyper::server::conn::http1::Builder;

pub struct ProxyChainServer;

#[allow(async_fn_in_trait)]
pub trait ProxyChain {
    fn is_request_qualified(
        _req: &Request<Incoming>,
        _filter_domains: Vec<String>,
    ) -> Result<(), Response<BoxBody<Bytes, hyper::Error>>> {
        Ok(())
    }

    async fn proxy(
        req: Request<Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>;

    async fn tunnel(upgraded: Upgraded, destination_addr: String) -> std::io::Result<()>;
}

impl ProxyChainServer {
    pub fn run(self, port: u16, notifier: Arc<tokio::sync::Notify>) {
        tokio::spawn(async move {
            self.main_loop(port, notifier)
                .await
                .expect("Main loop failed");
        });
    }

    async fn main_loop(
        &self,
        port: u16,
        notifier: Arc<tokio::sync::Notify>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        let listener = TcpListener::bind(addr).await?;
        notifier.notify_waiters();
        info!("Proxy chain listening on: {}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            tokio::task::spawn(async move {
                if let Err(err) = ServerBuilder::new()
                    .preserve_header_case(true)
                    .title_case_headers(true)
                    .serve_connection(
                        io,
                        service_fn(move |req| async move {
                            if let Err(e) = Self::is_request_qualified(
                                &req,
                                CONFIG.data_bright.allow_domains.clone(),
                            ) {
                                return Ok(e);
                            }

                            Self::proxy(req).await
                        }),
                    )
                    .with_upgrades()
                    .await
                {
                    error!("Failed to serve connection: {:?}", err);
                }
            });
        }
    }
}

impl ProxyChain for ProxyChainServer {
    fn is_request_qualified(
        req: &Request<Incoming>,
        filter_domains: Vec<String>,
    ) -> Result<(), Response<BoxBody<Bytes, hyper::Error>>> {
        // Only try to establish a proxy connection for requests with the CONNECT method
        // and defined destination host existing on a filter list
        if req.method() != Method::CONNECT {
            debug!("METHOD not allowed: {}", req.method());
            return Err(response(
                "CONNECT is only allowed method",
                StatusCode::METHOD_NOT_ALLOWED,
            ));
        }

        let authority = match req.uri().authority() {
            Some(authority) if authority.port().is_some() => authority,
            _ => {
                debug!("HOST and PORT must exist");
                return Err(response(
                    "HOST and PORT must exist",
                    StatusCode::BAD_REQUEST,
                ));
            }
        };

        if !filter_domains.contains(&authority.host().to_string()) {
            debug!("DOMAIN not proxyable: {}", authority.host());
            return Err(response("DOMAIN not proxyable", StatusCode::BAD_REQUEST));
        }

        Ok(())
    }

    async fn proxy(
        req: Request<Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        let destination_addr = req.uri().authority().expect("unreachable").to_string();

        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = Self::tunnel(upgraded, destination_addr).await {
                        error!("server io error: {}", e);
                    }
                }
                Err(e) => error!("upgrade error: {}", e),
            }
        });

        Ok(Response::new(empty()))
    }

    async fn tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<()> {
        let mut proxy_stream = TcpStream::connect(format!(
            "{}:{}",
            CONFIG.data_bright.host, CONFIG.data_bright.port
        ))
        .await?;
        let mut upgraded = TokioIo::new(upgraded);

        let proxy_auth = base64::engine::general_purpose::STANDARD.encode(format!(
            "{}:{}",
            CONFIG.data_bright.user, CONFIG.data_bright.password
        ));
        let proxy_auth_header_value = format!("Basic {}", proxy_auth);

        let connect_req = format!(
            "CONNECT {} HTTP/1.1\r\nHost: {}\r\nProxy-Authorization: {}\r\n\r\n",
            addr, addr, proxy_auth_header_value,
        );
        AsyncWriteExt::write_all(&mut proxy_stream, connect_req.as_bytes()).await?;
        proxy_stream.flush().await?;

        let mut buf = [0; 1024];
        let n = proxy_stream.read(&mut buf).await?;
        let response = String::from_utf8_lossy(&buf[..n]);
        if !response.contains("200") {
            return Err(std::io::Error::other(format!(
                "Proxy did not establish connection: {}",
                &response[0..12]
            )));
        }

        tokio::io::copy_bidirectional(&mut upgraded, &mut proxy_stream).await?;
        Ok(())
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

fn response<T: Into<Bytes>>(
    message: T,
    status: StatusCode,
) -> Response<BoxBody<Bytes, hyper::Error>> {
    let mut resp = Response::new(full(message));
    *resp.status_mut() = status;
    resp
}
