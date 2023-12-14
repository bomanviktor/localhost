mod common;
mod test_config {
    use localhost::server_config::server_config;
    #[test]
    fn test_fields() {
        let configs = server_config();
        for c in configs {
            assert!(!c.host.is_empty());
            assert!(!c.ports.is_empty());
            assert!(!c.default_error_paths.is_empty());
            assert!(c.body_size_limit > 0);
            assert!(!c.routes.is_empty());
        }
    }
}

mod test_requests {
    use crate::common::setup;
    use crate::test_requests::utils::get_status;
    use curl::easy::Easy;
    use http::StatusCode;
    use std::thread;
    mod utils {
        use super::*;
        pub fn get_status(header: &str) -> StatusCode {
            let status = header.split_whitespace().collect::<Vec<&str>>()[1];
            match status {
                "200" => StatusCode::OK,
                "308" => StatusCode::PERMANENT_REDIRECT,
                "400" => StatusCode::BAD_REQUEST,
                "401" => StatusCode::UNAUTHORIZED,
                "403" => StatusCode::FORBIDDEN,
                "404" => StatusCode::NOT_FOUND,
                "405" => StatusCode::METHOD_NOT_ALLOWED,
                "411" => StatusCode::LENGTH_REQUIRED,
                "413" => StatusCode::PAYLOAD_TOO_LARGE,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
    }
    mod valid {
        use super::*;
        #[test]
        fn test_get() {
            thread::spawn(|| setup());
            // Request 1
            let mut easy = Easy::new();
            easy.url("localhost:8080/path1").unwrap();
            easy.get(true).unwrap();
            // Set a closure to handle the response
            easy.header_function(|header| {
                // Process each header line (here, we print it to stdout)
                let header = String::from_utf8(header.to_vec()).unwrap_or_default();
                if header.contains("HTTP/1.1") {
                    assert_eq!(get_status(&header), StatusCode::OK);
                }
                true
            })
            .unwrap();
            easy.perform().unwrap_or_default();

            // Request 2
            let mut easy = Easy::new();
            easy.url("localhost:8080/path2").unwrap();
            easy.get(true).unwrap();
            // Set a closure to handle the response
            easy.header_function(|header| {
                // Process each header line (here, we print it to stdout)
                let header = String::from_utf8(header.to_vec()).unwrap_or_default();
                if header.contains("HTTP/1.1") {
                    assert_eq!(get_status(&header), StatusCode::OK);
                }
                true
            })
            .unwrap();
            easy.perform().unwrap_or_default();
        }
    }
}
