use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HarFile {
    log: Log,
}

impl HarFile {
    pub fn search(&self, request: &hyper::Request<hyper::Body>) -> Option<&Entry> {
        self.log.entries.iter().find(|entry| entry.matches(request))
    }
}

#[derive(Debug, Deserialize)]
struct Log {
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    request: Request,
    response: Response,
}

impl Entry {
    pub fn matches(&self, request: &hyper::Request<hyper::Body>) -> bool {
        return self.request.url.contains(request.uri().path())
            && self.request.method == request.method().as_str();
    }
}

impl From<&Entry> for hyper::Response<hyper::Body> {
    fn from(entry: &Entry) -> Self {
        hyper::Response::builder()
            .status(entry.response.status)
            .body(hyper::Body::from(entry.response.content.text.clone()))
            .unwrap()
    }
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
    text: String,
}
