use crate::server::{update_cookie, validate_cookie};
pub use crate::server_config::*;
pub fn server_config() -> Vec<ServerConfig<'static>> {
    vec![ServerConfig {
        host: "127.0.0.1",
        ports: vec![8080, 8081, 8082],
        default_error_path: None,
        body_size_limit: 1024,
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
                settings: None,
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
