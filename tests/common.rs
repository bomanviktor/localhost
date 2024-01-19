use localhost::log::init_logs;
use localhost::server::{content_type, servers, start};
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use std::fs::File;
use std::io::Read;
pub fn setup() {
    init_logs();
    start(servers());
}

pub fn send_request(
    client: &Client,
    url: &str,
    body: Vec<u8>,
    method: http::Method,
) -> reqwest::blocking::Response {
    let mut request_builder = match method {
        // UNSAFE
        http::Method::POST => client.post(url),
        http::Method::PUT => client.put(url),
        http::Method::PATCH => client.patch(url),
        http::Method::DELETE => client.delete(url),
        // SAFE
        http::Method::GET => client.get(url),
        http::Method::HEAD => client.head(url),
        _ => client.get(url),
    };

    request_builder = request_builder
        .header(CONTENT_TYPE, content_type(url))
        .body(body);

    let response = request_builder.send().unwrap();
    response
}

pub fn buffer_and_client(path: &str) -> (Vec<u8>, Client) {
    let mut file = File::open(path).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let client = Client::new();
    (buf, client)
}
