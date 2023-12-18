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

#[allow(dead_code)]
mod test_requests {
    use http::StatusCode;
    mod utils {
        use super::*;
        use localhost::client::utils::get_split_index;
        use std::str::FromStr;
        pub fn get_status(header: &str) -> StatusCode {
            let status = get_split_index(header, 1);
            StatusCode::from_str(status).unwrap_or(StatusCode::OK)
        }
    }
    /*
    mod valid {
        use super::*;
        #[test]
        fn test_get() {
            thread::spawn(setup);
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
             */
}
