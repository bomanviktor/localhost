use config::route::Settings;
use http::StatusCode;
use std::collections::HashMap;

use crate::server::{cookie_demo, update_cookie, validate_cookie, Cgi};
pub use crate::server_config::*;

// Function to configure the server settings
pub fn server_config() -> Vec<ServerConfig<'static>> {
    vec![ServerConfig {
        // IP address where the server listens.
        // Change "127.0.0.1" to your server's IP address if needed.
        host: "127.0.0.1",

        // Ports on which the server will listen. Add or remove ports as required.
        ports: vec![8080, 8081, 8082],

        // Path for custom error pages. Set to 'Some(path)' to enable, or leave as 'None' for default error handling.
        custom_error_path: None,

        // Maximum allowed size for request bodies in bytes. Adjust according to your needs.
        body_size_limit: 1000000000024,

        // Configuration for individual routes on the server.
        routes: vec![
            Route {
                // Path for the route. Adjust this to match the endpoint you wish to configure.
                url_path: "/api/update-cookie",
                // HTTP methods allowed for this route. Add or remove methods as needed.
                methods: vec![http::Method::POST],
                // Handler function for the route. Change 'update_cookie' to your custom function if required.
                handler: Some(update_cookie),
                // Route-specific settings. Leave as 'None' for default settings.
                settings: None,
            },
            // Additional routes follow the same structure. Customize each route as needed.
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
                handler: None, // No specific handler means processing is defined by 'settings'.
                settings: Some(Settings {
                    // Configuration for CGI scripts.
                    cgi_def: Some(HashMap::from([
                        // Map file extensions to CGI handlers. Add or remove mappings as required.
                        ("js", Cgi::JavaScript),
                        ("php", Cgi::PHP),
                        ("py", Cgi::Python),
                        ("rb", Cgi::Ruby),
                    ])),
                    // Enable directory listing for this route. Set to 'false' to disable.
                    list_directory: true,
                    // Additional CGI settings can be configured here.
                    // Leave as 'None' for defaults or specify to customize behavior.
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: None,
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
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
