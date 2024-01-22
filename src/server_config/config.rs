use config::route::Settings;
use http::StatusCode;
use std::collections::HashMap;

use crate::server::{cookie_demo, update_cookie, validate_cookie, Cgi};
pub use crate::server_config::*;
pub fn server_config() -> Vec<ServerConfig<'static>> {
    vec![ServerConfig {
        host: "localhost",
        ports: vec![8080, 8081, 8082],
        default_error_path: Some("/files/default_errors"),
        body_size_limit: 1000000000024,
        routes: vec![
            Route {
                url_path: "/api/update-cookie",
                methods: vec![http::Method::POST],
                handler: Some(update_cookie),
                settings: None,
            },
            Route {
                url_path: "/api/get-cookie",
                methods: vec![http::Method::GET],
                handler: Some(validate_cookie),
                settings: None,
            },
            Route {
                url_path: "/api/cookie-demo",
                methods: vec![http::Method::GET],
                handler: Some(cookie_demo),
                settings: None,
            },
            Route {
                url_path: "/cgi",
                methods: vec![http::Method::GET],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: None,
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: Some(HashMap::from([
                        ("js", Cgi::JavaScript),
                        ("php", Cgi::PHP),
                        ("py", Cgi::Python),
                        ("rb", Cgi::Ruby),
                    ])),
                    list_directory: true,
                }),
            },
            Route {
                url_path: "/test.txt",
                methods: vec![http::Method::GET, http::Method::POST],
                handler: None,
                settings: Some(Settings {
                    http_redirections: Some(vec!["/redirection-test"]),
                    redirect_status_code: Some(StatusCode::from_u16(301).unwrap()),
                    root_path: Some("/files"),
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/mega-dir",
                methods: vec![http::Method::GET],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: Some("/files"),
                    default_if_url_is_dir: Some("/dir.html"),
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/src",
                methods: vec![http::Method::GET],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: None,
                    default_if_url_is_dir: Some("/does-not-exist-mate"),
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/files",
                methods: vec![
                    http::Method::GET,
                    http::Method::POST,
                    http::Method::PUT,
                    http::Method::PATCH,
                    http::Method::DELETE,
                ],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: None,
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: true,
                }),
            },
        ],
    }]
}
