use std::str::FromStr;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HarFile {
    log: Log,
}

impl HarFile {
    pub fn search(&self, request: &hyper::Request<hyper::Body>, prefix: &str) -> Option<&Entry> {
        self.log
            .entries
            .iter()
            .find(|entry| entry.matches(request, prefix))
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
    pub fn matches(&self, request: &hyper::Request<hyper::Body>, prefix: &str) -> bool {
        let incoming_url = request.uri().path();
        let stored_url =
            hyper::Uri::from_str(&self.request.url).expect("Invalid URI in configuration");

        return incoming_url == stored_url.path().trim_start_matches(prefix)
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

#[cfg(test)]
mod tests {
    use hyper::{Body, Method};

    use super::{Content, Entry, Request, Response};

    const SOME_RESPONSE: Response = Response {
        status: 200,
        content: Content {
            text: String::new(),
        },
    };

    const PREFIX: &str = "/prefix";

    #[test]
    fn matching_takes_into_account_prefix() {
        let entry = Entry {
            request: Request {
                method: String::from("GET"),
                url: String::from("http://example.com:9000/prefix/api/v1/users"),
            },
            response: SOME_RESPONSE,
        };

        let request = hyper::Request::builder()
            .method(Method::GET)
            .uri("http://localhost:8080/api/v1/users")
            .body(Body::empty())
            .unwrap();

        assert!(entry.matches(&request, PREFIX));
    }

    #[test]
    fn matching_compares_the_full_path() {
        let entry = Entry {
            request: Request {
                method: String::from("GET"),
                url: String::from("http://example.com:9000/prefix/api/v1/users/identifier"),
            },
            response: SOME_RESPONSE,
        };

        let request = hyper::Request::builder()
            .method(Method::GET)
            .uri("http://localhost:8080/api/v1/users")
            .body(Body::empty())
            .unwrap();

        assert!(!entry.matches(&request, PREFIX));
    }
}
