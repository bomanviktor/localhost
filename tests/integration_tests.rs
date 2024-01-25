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
mod test_handle_client {
    use lazy_static::lazy_static;
    use std::thread;
    use std::time::Duration;
    const HOST: &str = "http://127.0.0.1:8080";
    lazy_static! {
        static ref CLIENT: Client = Client::new();
    }

    use super::*;
    use crate::common::send_request;
    use http::Method;
    use localhost::type_aliases::Bytes;
    use reqwest::blocking::Client;

    #[test]
    fn cgi_request() {
        setup();
        let valid_endpoint = "/cgi/php.php";
        let resp = send_request(
            &CLIENT,
            &format!("{HOST}{valid_endpoint}"),
            Bytes::new(),
            Method::GET,
        );

        assert_eq!(resp.status().as_u16(), 200);

        // Not found test
        let invalid_endpoint = "/cgi/php.kek";
        let resp = send_request(
            &CLIENT,
            &format!("{HOST}{invalid_endpoint}"),
            Bytes::new(),
            Method::GET,
        );

        assert_eq!(resp.status().as_u16(), 404);
    }

    #[test]
    fn bad_request() {
        setup();
        // Not found test
        let valid_endpoint = "/";
        let resp = send_request(
            &CLIENT,
            &format!("{HOST}{valid_endpoint}"),
            Bytes::new(),
            Method::GET,
        );

        assert!(resp.status().is_client_error());
    }

    #[test]
    fn not_found() {
        setup();
        // Not found test
        let invalid_endpoint = "/oga-boga";
        let resp = send_request(
            &CLIENT,
            &format!("{HOST}{invalid_endpoint}"),
            Bytes::new(),
            Method::GET,
        );

        assert!(resp.status().is_client_error());
    }

    #[test]
    fn redirections() {
        setup();
        // Not found test
        let valid_endpoint = "/redirection-test";
        let resp = send_request(
            &CLIENT,
            &format!("{HOST}{valid_endpoint}"),
            Bytes::new(),
            Method::GET,
        );

        assert!(resp.status().is_success());
    }

    #[test]
    fn handlers() {
        setup();
        // Not found test
        let valid_endpoint = "/api/update-cookie";
        let resp = send_request(
            &CLIENT,
            &format!("{HOST}{valid_endpoint}"),
            Bytes::new(),
            Method::POST,
        );

        assert!(resp.status().is_success());
        thread::sleep(Duration::from_secs(1));

        // Attempt to get a cookie we don't have.
        let invalid_endpoint = "/api/get-cookie";
        let resp = send_request(
            &CLIENT,
            &format!("{HOST}{invalid_endpoint}"),
            Bytes::new(),
            Method::GET,
        );

        assert!(resp.status().is_client_error());
    }

    #[test]
    #[ignore]
    fn default_file_exists() {
        setup();
        // Not found test
        let valid_endpoint = "/mega-dir";
        let resp = send_request(
            &CLIENT,
            &format!("{HOST}{valid_endpoint}"),
            Bytes::new(),
            Method::GET,
        );

        assert!(resp.status().is_success());
    }

    #[test]
    fn directory_listing() {
        setup();
        // Not found test
        let valid_endpoint = "/files";
        let resp = send_request(
            &CLIENT,
            &format!("{HOST}{valid_endpoint}"),
            Bytes::new(),
            Method::GET,
        );

        assert!(resp.status().is_success());
    }

    mod binary_file {
        use super::*;
        use crate::common::send_request;
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
            use crate::common::get_buffer;
            #[test]
            fn post() {
                setup();
                let buf = get_buffer("./files/tests/test.png");
                let valid_endpoint = "/files/tests.png";

                let response = send_request(
                    &CLIENT,
                    &format!("{HOST}{valid_endpoint}"),
                    buf.clone(),
                    http::Method::POST,
                );

                check_response(true, response, buf);
            }

            #[test]
            fn put() {
                setup();
                let buf = get_buffer("./files/tests/test.png");
                let valid_endpoint = "/files/tests.png";

                // Post and Delete
                let response = send_request(
                    &CLIENT,
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
        use reqwest::header::{CONTENT_TYPE, TRANSFER_ENCODING};

        fn send_chunked_request(
            client: &Client,
            url: &str,
            body: &'static str,
            method: Method,
        ) -> reqwest::blocking::Response {
            let mut request_builder = match method {
                Method::POST => client.post(url),
                Method::GET => client.get(url),
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
                setup();

                let body = "Wiki\r\npedia\r\n in\r\n\r\nchunks.\r\n\r\n";

                let valid_endpoint = "/test.txt";

                let response = send_chunked_request(
                    &CLIENT,
                    &format!("{HOST}{valid_endpoint}"),
                    body,
                    Method::POST,
                );

                // Check the response status and body
                assert_eq!(response.status(), reqwest::StatusCode::OK);
                assert_eq!(response.bytes().unwrap_or_default(), body);
            }

            #[test]
            fn invalid() {}
        }
    }
}
