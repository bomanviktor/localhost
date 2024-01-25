use lazy_static::lazy_static;
use localhost::log;
use localhost::log::{init_logs, LogFileType};
use localhost::server::{content_type, start};
use localhost::server_config::server_config;
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use std::fs::File;
use std::io::Read;
use std::sync::{Mutex, Once};
use std::thread;

lazy_static! {
    static ref SERVER_STARTED: Mutex<()> = Mutex::new(());
    static ref SERVER_INITIALIZATION: Once = Once::new();
}

pub fn setup() {
    SERVER_INITIALIZATION.call_once(|| {
        // Start the server only once
        let _ = thread::spawn(|| {
            init_logs();
            start(server_config());
        });
    });

    // Wait for the server to start (this is a simplistic example, adjust as needed)
    thread::sleep(std::time::Duration::from_secs(1));
    println!("Server setup completed");
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

    let debug_info = request_builder.try_clone().expect("Body is a stream.");
    let response = match request_builder.send() {
        Ok(r) => r,
        Err(e) => {
            log!(
                LogFileType::Server,
                format!("Test failed {e}. Request builder: {debug_info:?}")
            );
            panic!("Test failed {e}. Request builder: {debug_info:?}");
        }
    };
    response
}
pub fn get_buffer(path: &str) -> Vec<u8> {
    let mut file = File::open(path).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}
