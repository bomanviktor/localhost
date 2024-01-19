const HOST: &str = "http://127.0.0.1:8080";
use common::setup;
use reqwest;

mod common;

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref TEST_SERVER: Mutex<()> = {
        let _lock = Mutex::new(());
        setup();
        _lock
    };
}

mod test_server {
    use super::*;
    #[test]
    fn server() {
        // This line ensures that the setup function is called once and only once
        let _ = *TEST_SERVER.lock().unwrap();
    }
}

mod test_config {
    use localhost::server_config::server_config;
    #[test]
    fn test_fields() {
        let configs = server_config();
        for c in configs {
            assert!(!c.host.is_empty());
            assert!(!c.ports.is_empty());
            assert!(c.body_size_limit > 0);
            assert!(!c.routes.is_empty());
        }
    }
}

mod binary_file {
    use super::*;
    use crate::common::{buffer_and_client, send_request};
    use localhost::type_aliases::Bytes;

    fn check_response(valid: bool, response: reqwest::Response, buf: Bytes) {
        if valid {
            assert_eq!(response.status(), reqwest::StatusCode::OK);
            assert!(response.content_length().unwrap() > buf.len() as u64);
        } else {
            assert_ne!(response.status(), reqwest::StatusCode::OK);
        }
    }
    mod valid {
        use super::*;
        #[tokio::test]
        async fn post() {
            let (buf, client) = buffer_and_client("./files/test.png").await;
            let valid_endpoint = "/files/tests/test.png";

            // Make sure the file does not exist.
            send_request(
                &client,
                &format!("{HOST}{valid_endpoint}"),
                buf.clone(),
                http::Method::DELETE,
            )
            .await;

            // Post
            let response = send_request(
                &client,
                &format!("{HOST}{valid_endpoint}"),
                buf.clone(),
                http::Method::POST,
            )
            .await;

            check_response(true, response, buf);
        }

        #[tokio::test]
        async fn put() {
            let (buf, client) = buffer_and_client("./files/test.png").await;
            let valid_endpoint = "/files/tests/test.png";

            // Post and Delete
            let response = send_request(
                &client,
                &format!("{HOST}{valid_endpoint}"),
                buf.clone(),
                http::Method::PUT,
            )
            .await;

            check_response(true, response, buf);
        }
    }

    mod invalid {}
}

mod chunked_encoding {
    use super::*;
    use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE, TRANSFER_ENCODING};
    use reqwest::Client;

    pub async fn send_chunked_request(
        client: &Client,
        url: &str,
        body: &'static str,
        method: http::Method,
    ) -> reqwest::Response {
        let mut request_builder = match method {
            http::Method::POST => client.post(url),
            http::Method::GET => client.get(url),
            _ => client.get(url),
        };

        request_builder = request_builder
            .header(CONTENT_TYPE, "text/plain")
            .header(CONTENT_LENGTH, body.len())
            .header(TRANSFER_ENCODING, "chunked")
            .body(body);
        let response = request_builder.send().await;
        response.unwrap()
    }
}

#[cfg(test)]
mod request_testing {
    use super::*;
    use crate::chunked_encoding::send_chunked_request;
    use crate::HOST;

    #[tokio::test]
    async fn chunk_test() {
        let client = &reqwest::Client::new();
        let body = "Wiki\r\npedia\r\n in\r\n\r\nchunks.\r\n\r\n";
        let formatted_url = &format!("{HOST}/files/tests/test.txt");
        let response = send_chunked_request(client, formatted_url, body, http::Method::POST).await;
        assert_eq!(response.status(), reqwest::StatusCode::OK);
    }

    #[tokio::test]
    async fn valid() {
        let client = reqwest::Client::new();

        // Test: Sending a request with a cookie
        let res = client
            .get(format!("{}/api/get-cookie", HOST))
            .header("Cookie", "grit:lab-cookie=valid_cookie_value")
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), reqwest::StatusCode::OK);

        // Test: Sending request to valid endpoint /test
        let res = client
            .post(format!("{}/api/update-cookie", HOST))
            .header("Cookie", "grit:lab-cookie=invalid_cookie_value")
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), reqwest::StatusCode::OK);

        // Test: Sending request to valid endpoint /test
        let res = client
            .post(format!("{}/test.txt", HOST))
            .header("Cookie", "grit:lab-cookie=invalid_cookie_value")
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), reqwest::StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid() {
        let client = reqwest::Client::new();

        // Test: Sending request to invalid endpoint /wrong-path
        let res = client
            .get(format!("{}/wrong-path", HOST))
            .header("Cookie", "grit:lab-cookie=invalid_cookie_value")
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), reqwest::StatusCode::NOT_FOUND);

        // Sending a request without a cookie
        let res = client
            .get(format!("{}/api/get-cookie", HOST))
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), reqwest::StatusCode::UNAUTHORIZED);
    }

    use std::process::Command;
    #[tokio::test]
    async fn curl_testing() {
        let number_of_requests = 10;
        // let delay_between_requests = Duration::from_secs(1); // Delay between requests

        for _ in 0..number_of_requests {
            let command = match Command::new("curl")
                // Add method and headers as needed
                .arg("-X")
                .arg("POST")
                .arg("-H")
                .arg("Content-Type: text/plain")
                // Add body
                .arg("-d")
                .arg("Hello World!")
                .arg(format!("http://{}/", HOST))
                .output()
            {
                Ok(output) => {
                    println!("Status: {}", output.status);
                    println!("Output: {}", String::from_utf8_lossy(&output.stdout));
                    assert_eq!(output.status.code(), Some(6)); //Why is this 6?
                }
                Err(e) => eprintln!("Failed to execute process: {}", e),
            };
            println!("{:?}", command);

            // thread::sleep(delay_between_requests);
        }
    }
}

#[allow(dead_code)]
mod test_requests {
    use http::StatusCode;
    mod utils {
        use super::*;
        use localhost::server::utils::get_split_index;
        use std::str::FromStr;

        pub fn get_status(header: &str) -> StatusCode {
            let status = get_split_index(header, 1);
            StatusCode::from_str(status).unwrap_or(StatusCode::OK)
        }
    }
}

#[cfg(test)]
mod sessions_unit_tests {
    use http::header::COOKIE;
    use http::{HeaderValue, Request, StatusCode};
    use localhost::server::update_cookie;
    use std::collections::HashMap;

    #[test]
    fn test_update_cookie_existing() {
        use localhost::server_config::server_config;
        let configs = server_config();
        let mut headers = HashMap::new();
        headers.insert(COOKIE, HeaderValue::from_static("existing_cookie_value"));

        let req = Request::builder()
            .uri("/test")
            .header(COOKIE, "existing_cookie_value")
            .body(vec![])
            .unwrap();

        for config in configs {
            let result = update_cookie(&req, &config).unwrap();
            assert_eq!(result.status(), StatusCode::OK);
            assert!(!result.headers().contains_key(COOKIE));
        }
    }

    #[test]
    fn test_update_cookie_non_existing() {
        use localhost::server_config::server_config;
        let configs = server_config();
        let mut headers = HashMap::new();
        headers.insert(COOKIE, HeaderValue::from_static("grit:lab-cookie"));

        let req = Request::builder().uri("/test").body(vec![]).unwrap();

        for config in configs {
            let result = update_cookie(&req, &config).unwrap();
            assert_eq!(result.status(), StatusCode::OK);
            assert!(!result.headers().contains_key(COOKIE));
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::HOST;
    #[tokio::test]
    async fn test_validate_cookie_endpoint() {
        let client = reqwest::Client::new();

        // Sending a request with a cookie
        let res = client
            .get(format!("{}/api/get-cookie", HOST))
            .header("Cookie", "grit:lab-cookie=valid_cookie_value")
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), reqwest::StatusCode::OK);

        // Sending a request without a cookie
        let res = client
            .get(format!("{}/api/get-cookie", HOST))
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), reqwest::StatusCode::UNAUTHORIZED);
        // Additional assertions can be added here based on response content
    }
}
