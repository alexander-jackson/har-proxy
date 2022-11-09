use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Response, Server};
use tracing_subscriber::{filter::targets::Targets, layer::SubscriberExt, util::SubscriberInitExt};

mod args;
mod structure;

use crate::args::Args;
use crate::structure::HarFile;

fn setup() {
    let log_config = env::var("RUST_LOG");
    let filter_layer = Targets::from_str(log_config.as_deref().unwrap_or("info")).unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup();

    let args = Args::parse()?;

    let raw = std::fs::read_to_string(&args.proxy_from)?;
    let harfile: Arc<HarFile> = Arc::new(serde_json::from_str(&raw)?);

    tracing::info!(
        "Proxying requsts from archive file at {:?}",
        args.proxy_from
    );

    let service = make_service_fn(move |_| {
        let spec = harfile.clone();

        async move {
            Ok::<_, Error>(service_fn(move |req: hyper::Request<Body>| {
                handle_request(req, spec.clone())
            }))
        }
    });

    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, args.port).into();
    tracing::info!("Binding server on port {}", addr);

    let server = Server::bind(&addr).serve(service);

    server.await?;

    Ok(())
}

async fn handle_request(
    request: hyper::Request<Body>,
    spec: Arc<HarFile>,
) -> Result<hyper::Response<Body>, Error> {
    tracing::info!(method = ?request.method(), uri = ?request.uri(), "Handling a request");

    let response = spec.search(&request).map_or_else(not_found, Into::into);

    Ok(response)
}

fn not_found() -> Response<Body> {
    Response::builder().status(404).body(Body::empty()).unwrap()
}
