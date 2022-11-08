use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Server};
use serde::Deserialize;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let raw = std::fs::read_to_string("releases.har")?;
    let parsed: Arc<HarFile> = Arc::new(serde_json::from_str(&raw)?);

    let make_svc = make_service_fn(move |_| {
        let spec = parsed.clone();
        async move {
            Ok::<_, Error>(service_fn(move |req: hyper::Request<Body>| {
                let spec = spec.clone();

                handle_request(req, spec)
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
    let path = request.uri().path();

    // Find a corresponding entry
    let entry = spec
        .log
        .entries
        .iter()
        .find(|entry| entry.request.url.contains(path));

    let response = match entry {
        Some(value) => hyper::Response::builder()
            .status(value.response.status)
            .body(Body::from(value.response.content.text.clone()))
            .unwrap(),
        None => hyper::Response::builder()
            .status(404)
            .body(Body::empty())
            .unwrap(),
    };

    Ok(response)
}

#[derive(Debug, Deserialize)]
struct HarFile {
    log: HarFileInner,
}

#[derive(Debug, Deserialize)]
struct HarFileInner {
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    request: Request,
    response: Response,
}

#[derive(Debug, Deserialize)]
struct Request {
    method: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct Response {
    status: u16,
    content: Content,
}

#[derive(Debug, Deserialize)]
struct Content {
    size: usize,
    text: String,
}
