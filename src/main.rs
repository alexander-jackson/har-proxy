use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Response, Server};

mod structure;

use crate::structure::HarFile;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let raw = std::fs::read_to_string("releases.har")?;
    let harfile: Arc<HarFile> = Arc::new(serde_json::from_str(&raw)?);

    let make_svc = make_service_fn(move |_| {
        let spec = harfile.clone();

        async move {
            Ok::<_, Error>(service_fn(move |req: hyper::Request<Body>| {
                handle_request(req, spec.clone())
            }))
        }
    });

    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 10320).into();
    let server = Server::bind(&addr).serve(make_svc);

    server.await?;

    Ok(())
}

async fn handle_request(
    request: hyper::Request<Body>,
    spec: Arc<HarFile>,
) -> Result<hyper::Response<Body>, Error> {
    Ok(spec.search(&request).map_or_else(not_found, Into::into))
}

fn not_found() -> Response<Body> {
    Response::builder().status(404).body(Body::empty()).unwrap()
}
