use http::{Method, Request, StatusCode};
use localhost::server::Cgi;
use localhost::server_config::route::{Route, Settings};
use localhost::server_config::ServerConfig;
use localhost::type_aliases::Bytes;
use std::collections::HashMap;

#[allow(dead_code)]

// Mock functions and data for testing
pub fn mock_route() -> Route<'static> {
    let route = Route {
        methods: vec![
            Method::GET,
            Method::OPTIONS,
            Method::HEAD,
            Method::TRACE,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            // Method::CONNECT, // Excluded as it's unimplemented
        ],
        url_path: "/",
        handler: None,
        settings: None,
    };
    route
}

#[allow(dead_code)]

pub fn mock_request(
    method: Method,
    path: &str,
    body: Option<&str>,
    headers: Option<Vec<(&str, &str)>>,
) -> Request<Bytes> {
    let mut req = Request::builder()
        .method(method)
        .uri(format!("http://localhost:8080{path}"));

    for (key, value) in headers.unwrap_or_default() {
        req = req.header(key, value);
    }

    req.body(Bytes::from(body.unwrap_or_default())).unwrap()
}

pub fn mock_server_config() -> ServerConfig<'static> {
    let config = ServerConfig {
        host: "127.0.0.1",
        ports: vec![8080],
        custom_error_path: None,
        body_size_limit: 10024,
        routes: vec![
            Route {
                url_path: "/cgi",
                methods: vec![Method::GET],
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
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/test",
                methods: vec![Method::GET],
                handler: None,
                settings: None,
            },
            Route {
                url_path: "/test.txt",
                methods: vec![
                    Method::GET,
                    Method::POST,
                    Method::HEAD,
                    Method::OPTIONS,
                    Method::TRACE,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: Some("/files"),
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/test_put.txt",
                methods: vec![
                    Method::GET,
                    Method::POST,
                    Method::HEAD,
                    Method::OPTIONS,
                    Method::TRACE,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: Some("/files"),
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/patch_test.txt",
                methods: vec![
                    Method::GET,
                    Method::POST,
                    Method::HEAD,
                    Method::OPTIONS,
                    Method::TRACE,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: Some("/files"),
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/delete_test.txt",
                methods: vec![
                    Method::GET,
                    Method::POST,
                    Method::HEAD,
                    Method::OPTIONS,
                    Method::TRACE,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: Some("/files"),
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/non_existing_file.txt",
                methods: vec![
                    Method::GET,
                    Method::POST,
                    Method::HEAD,
                    Method::OPTIONS,
                    Method::TRACE,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: Some("/files"),
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/tests/redirect.txt",
                methods: vec![Method::GET],
                handler: None,
                settings: Some(Settings {
                    http_redirections: Some(vec!["/redirection"]),
                    redirect_status_code: Some(StatusCode::TEMPORARY_REDIRECT),
                    root_path: Some("/files"),
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
        ],
    };
    config
}
