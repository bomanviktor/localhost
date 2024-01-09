const HOST: &str = "http://127.0.0.1:8080";
use common::setup;
use reqwest;
use std::thread;

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
            let valid_endpoint = "/test";

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
