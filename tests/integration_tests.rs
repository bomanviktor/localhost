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

use std::thread;

mod binary_file {
    use super::*;
    use crate::common::{buffer_and_client, send_request, setup};
    use localhost::type_aliases::Bytes;

    fn check_response(valid: bool, response: reqwest::blocking::Response, buf: Bytes) {
        if valid {
            assert_eq!(response.status(), reqwest::StatusCode::OK);
            assert!(response.content_length().unwrap() > buf.len() as u64);
        } else {
            assert_ne!(response.status(), reqwest::StatusCode::OK);
        }
    }
    mod valid {
        use super::*;
        #[test]
        fn post() {
            thread::spawn(setup);
            let (buf, client) = buffer_and_client("./files/tests/test.png");
            let valid_endpoint = "/files/tests.png";

            // Make sure the file does not exist.
            send_request(
                &client,
                &format!("{HOST}{valid_endpoint}"),
                buf.clone(),
                http::Method::DELETE,
            );

            // Post
            let response = send_request(
                &client,
                &format!("{HOST}{valid_endpoint}"),
                buf.clone(),
                http::Method::POST,
            );

            check_response(true, response, buf);
        }

        #[test]
        fn put() {
            thread::spawn(setup);
            let (buf, client) = buffer_and_client("./files/tests/test.png");
            let valid_endpoint = "/files/tests.png";

            // Post and Delete
            let response = send_request(
                &client,
                &format!("{HOST}{valid_endpoint}"),
                buf.clone(),
                http::Method::PUT,
            );

            check_response(true, response, buf);
        }
    }

    mod invalid {}
}

mod chunked_encoding {
    use super::*;
    use reqwest::blocking::Client;
    use reqwest::header::{CONTENT_TYPE, TRANSFER_ENCODING};

    fn send_chunked_request(
        client: &Client,
        url: &str,
        body: &'static str,
        method: http::Method,
    ) -> reqwest::blocking::Response {
        let mut request_builder = match method {
            http::Method::POST => client.post(url),
            http::Method::GET => client.get(url),
            _ => client.get(url),
        };

        request_builder = request_builder
            .header(CONTENT_TYPE, "text/plain")
            .header(TRANSFER_ENCODING, "chunked")
            .body(body);

        let response = request_builder.send().unwrap();
        response
    }

    #[cfg(test)]
    mod request_testing {
        use super::*;
        use crate::HOST;

        #[test]
        fn chunk_test() {
            let body = "Wiki\r\npedia\r\n in\r\n\r\nchunks.\r\n\r\n";

            let client = Client::new();

            let valid_endpoint = "/files";

            let response = send_chunked_request(
                &client,
                &format!("{HOST}{valid_endpoint}"),
                body,
                http::Method::GET,
            );

            // Check the response status and body
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

            // Test: Send request with with body exceeding 10024 bytes
            //Body with size larger than 10024 bytes
            let mut body = String::from("a");
            for _ in 0..20000 {
                body.push('a');
            }

            println!("Body length is {}", body.len());

            let res = client
                .get(format!("{}/test", HOST))
                .header("Cookie", "grit:lab-cookie=invalid_cookie_value")
                .body(body)
                .send()
                .await
                .unwrap();

            assert_eq!(res.status(), reqwest::StatusCode::PAYLOAD_TOO_LARGE);
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
            .body(String::new())
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

        let req = Request::builder().uri("/test").body(String::new()).unwrap();

        for config in configs {
            let result = update_cookie(&req, &config).unwrap();
            assert_eq!(result.status(), StatusCode::OK);
            assert!(!result.headers().contains_key(COOKIE));
        }
    }
}
