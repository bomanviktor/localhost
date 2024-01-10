use std::collections::HashMap;

use config::route::Settings;

use crate::server::{update_cookie, validate_cookie};
pub use crate::server_config::*;
pub fn server_config() -> Vec<ServerConfig<'static>> {
    vec![ServerConfig {
        host: "127.0.0.1",
        ports: vec![8080, 8081, 8082],
        default_error_path: None,
        body_size_limit: 10024,
        routes: vec![
            Route {
                path: "/api/update-cookie",
                methods: vec![http::Method::POST],
                handler: Some(update_cookie),
                settings: None,
            },
            Route {
                path: "/api/get-cookie",
                methods: vec![http::Method::GET],
                handler: Some(validate_cookie),
                settings: None,
            },
            Route {
                path: "/test.png",
                methods: vec![http::Method::GET],
                handler: None,
                settings: Some(Settings {
                    http_redirections: vec![],
                    redirect_status_code: http::StatusCode::MOVED_PERMANENTLY,
                    root_path: None,
                    default_if_url_is_dir: "index.html",
                    default_if_request_is_dir: "index.html",
                    cgi_def: HashMap::new(),
                    list_directory: false,
                }),
            },
            Route {
                path: "/files", // this is does not allow files/* to be accessed
                methods: vec![http::Method::GET],
                handler: None,
                //test directory listing
                settings: Some(Settings {
                    http_redirections: vec![],
                    redirect_status_code: http::StatusCode::MOVED_PERMANENTLY,
                    root_path: Some("./src/"), // not really working yet
                    default_if_url_is_dir: "index.html", //WIP
                    default_if_request_is_dir: "index.html", //WIP
                    cgi_def: HashMap::new(),
                    list_directory: true,
                }),
            },
            Route {
                path: "/test",
                methods: vec![http::Method::GET, http::Method::POST],
                handler: None,
                settings: None,
            },
        ],
    }]
}
