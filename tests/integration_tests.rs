const HOST: &str = "http://127.0.0.1:8080";
use common::setup;
use reqwest;

mod common;
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

    mod get {
        #[test]
        fn valid() {}

        #[test]
        fn invalid() {}
    }

    mod post {
        use super::*;
        #[test]
        fn valid() {
            thread::spawn(setup);

            let body = "Wiki\r\npedia\r\n in\r\n\r\nchunks.\r\n\r\n";

            let client = Client::new();
            let valid_endpoint = "/test.txt";

            let response = send_chunked_request(
                &client,
                &format!("{HOST}{valid_endpoint}"),
                body,
                http::Method::POST,
            );

            // Check the response status and body
            assert_eq!(response.status(), reqwest::StatusCode::OK);
            assert_eq!(response.bytes().unwrap_or_default(), body);
        }

        #[test]
        fn invalid() {}
    }
}
