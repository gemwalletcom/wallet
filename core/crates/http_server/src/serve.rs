use std::convert::Infallible;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use axum::ServiceExt;
use axum::response::Response;
use gem_tracing::{error_fields, info_with_fields};
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use hyper_util::server::conn::auto::Builder;
use hyper_util::server::graceful::GracefulShutdown;
use hyper_util::service::TowerToHyperService;
use tokio::net::TcpListener;
use tower::Service;

use crate::shutdown::ShutdownReceiver;

#[derive(Clone, Copy)]
pub struct ServeConfig {
    pub header_read_timeout: Duration,
    pub grace: Duration,
}

pub async fn serve<S>(service: S, address: SocketAddr, config: ServeConfig, mut shutdown: ShutdownReceiver) -> Result<(), Box<dyn Error + Send + Sync>>
where
    S: Service<http::Request<Incoming>, Response = Response, Error = Infallible> + Clone + Send + Sync + 'static,
    S::Future: Send,
{
    let listener = TcpListener::bind(address).await.map_err(|error| format!("failed to bind {address}: {error}"))?;
    let mut make_service = service.into_make_service_with_connect_info::<SocketAddr>();
    let mut builder = Builder::new(TokioExecutor::new());
    builder.http1().timer(TokioTimer::new()).header_read_timeout(config.header_read_timeout);
    let graceful = GracefulShutdown::new();
    info_with_fields!("server started", address = address.to_string());

    while !*shutdown.borrow() {
        let accepted = tokio::select! {
            accepted = listener.accept() => accepted,
            _ = shutdown.changed() => break,
        };
        let (stream, remote) = match accepted {
            Ok(accepted) => accepted,
            Err(error) if is_connection_error(&error) => continue,
            Err(error) => {
                error_fields!("server accept failed", error = error.to_string());
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };
        std::future::poll_fn(|cx| Service::<SocketAddr>::poll_ready(&mut make_service, cx)).await?;
        let service = Service::<SocketAddr>::call(&mut make_service, remote).await?;
        let connection = builder.serve_connection_with_upgrades(TokioIo::new(stream), TowerToHyperService::new(service)).into_owned();
        let connection = graceful.watch(connection);
        tokio::spawn(async move {
            let _ = connection.await;
        });
    }

    drop(listener);
    info_with_fields!("server draining", grace = config.grace.as_secs());
    let _ = tokio::time::timeout(config.grace, graceful.shutdown()).await;
    Ok(())
}

fn is_connection_error(error: &io::Error) -> bool {
    matches!(error.kind(), io::ErrorKind::ConnectionRefused | io::ErrorKind::ConnectionAborted | io::ErrorKind::ConnectionReset)
}
