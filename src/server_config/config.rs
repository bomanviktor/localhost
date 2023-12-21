use crate::server::Cgi;
pub use crate::server_config::*;
use http::StatusCode;
use std::collections::HashMap;

pub fn server_config() -> Vec<ServerConfig<'static>> {
    vec![ServerConfig {
        host: "127.0.0.1",
        ports: vec![8080, 8081, 8082],
        default_error_path: Some("src/default_errors/"),
        body_size_limit: 1024,
        routes: vec![Route {
            path: "/test.png",
            accepted_http_methods: vec![
                http::Method::GET,
                http::Method::POST,
                http::Method::OPTIONS,
                http::Method::DELETE,
            ],
            http_redirections: vec!["/test1"],
            redirect_status_code: StatusCode::PERMANENT_REDIRECT,
            root_path: Some("src"),
            default_if_url_is_dir: "some default",
            default_if_request_is_dir: "some other default",
            cgi_def: HashMap::from([
                ("php", Cgi::PHP),
                ("rb", Cgi::Ruby),
                ("py", Cgi::Python),
                ("js", Cgi::JavaScript),
                ("cpp", Cgi::Cpp),
            ]),
            list_directory: true,
        }],
    }]
}
