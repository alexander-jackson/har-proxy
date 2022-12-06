use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Method, Response, Server};
use tokio::sync::Mutex;
use tracing_subscriber::{filter::targets::Targets, layer::SubscriberExt, util::SubscriberInitExt};

mod args;
mod hot_reload;
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
    let prefix: Arc<str> = Arc::from(args.prefix.as_str());
    let path = args.proxy_from.as_path();

    let raw = std::fs::read_to_string(path)?;
    let archive: Arc<Mutex<HarFile>> = Arc::new(Mutex::new(serde_json::from_str(&raw)?));

    let hot_reload_archive = Arc::clone(&archive);
    let hot_reload_path = args.proxy_from.clone();

    // Setup auto-reloading
    tokio::spawn(async move {
        loop {
            let archive = Arc::clone(&hot_reload_archive);
            let path = hot_reload_path.clone();

            if let Err(error) = hot_reload::run(path, archive).await {
                tracing::error!("Got an error when hot reloading: {:?}", error);
            }
        }
    });

    tracing::info!("Proxying requsts from archive file at {:?}", path);

    let service = make_service_fn(move |_| {
        let spec = archive.clone();
        let prefix = prefix.clone();

        async move {
            Ok::<_, Error>(service_fn(move |req: hyper::Request<Body>| {
                handle_request(req, spec.clone(), prefix.clone())
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
    spec: Arc<Mutex<HarFile>>,
    prefix: Arc<str>,
) -> Result<hyper::Response<Body>, Error> {
    let method = request.method();
    let uri = request.uri();

    if method != Method::GET {
        tracing::warn!(
            "The proxy only supports GET requests, {} requests will be returned a 404",
            method
        );

        return Ok(not_found(&uri));
    }

    tracing::info!(?uri, "Handling a request");

    let spec = spec.lock().await;

    let response = spec
        .search(&request, prefix.as_ref())
        .map_or_else(|| not_found(&uri), Into::into);

    Ok(response)
}

fn not_found(uri: &hyper::Uri) -> Response<Body> {
    tracing::warn!("Failed to find entry for {}", uri);

    Response::builder().status(404).body(Body::empty()).unwrap()
}
