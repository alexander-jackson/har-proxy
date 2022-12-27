use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Error, Method, Response, Server, Uri};
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
    let prefix: Arc<Vec<String>> = Arc::from(args.prefixes);

    let raw = std::fs::read_to_string(&args.proxy_from)?;
    let archive: Arc<Mutex<HarFile>> = Arc::new(Mutex::new(serde_json::from_str(&raw)?));

    tracing::info!(
        "Proxying requsts from archive file at {:?}",
        args.proxy_from
    );

    // Setup auto-reloading
    hot_reload::spawn(args.proxy_from, Arc::clone(&archive));

    let service = make_service_fn(move |_| {
        let spec = Arc::clone(&archive);
        let prefix = prefix.clone();

        async move {
            Ok::<_, Error>(service_fn(move |req: hyper::Request<Body>| {
                handle_request(req, Arc::clone(&spec), prefix.clone())
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
    prefixes: Arc<Vec<String>>,
) -> Result<hyper::Response<Body>, Error> {
    let method = request.method();
    let uri = request.uri();

    if method != Method::GET {
        tracing::warn!(
            "The proxy only supports GET requests, {} requests will be returned a 404",
            method
        );

        return Ok(not_found(uri));
    }

    tracing::info!(?uri, "Handling a request");

    let spec = spec.lock().await;

    match spec.search(&request, &prefixes) {
        Some(res) => Ok(res.into()),
        None => proxy_to_base(uri).await,
    }
}

async fn proxy_to_base(uri: &Uri) -> Result<hyper::Response<Body>, Error> {
    let client = Client::new();
    let path = format!("http://localhost:20010{}", uri.path());

    tracing::info!("Proxing request to base at {}", path);

    client.get(Uri::from_str(&path).unwrap()).await
}

fn not_found(uri: &Uri) -> Response<Body> {
    tracing::warn!("Failed to find entry for {}", uri);

    Response::builder().status(404).body(Body::empty()).unwrap()
}
