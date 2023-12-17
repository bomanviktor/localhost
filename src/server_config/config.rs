use crate::server_config::route::cgi::Cgi;
pub use crate::server_config::*;
use http::StatusCode;
use std::collections::HashMap;

pub fn server_config() -> Vec<ServerConfig<'static>> {
    vec![ServerConfig {
        host: "127.0.0.1",
        ports: vec![8080, 8081, 8082],
        default_error_paths: HashMap::from([
            (StatusCode::BAD_REQUEST, "/400.html"),
            (StatusCode::FORBIDDEN, "/403.html"),
            (StatusCode::NOT_FOUND, "/404.html"),
            (StatusCode::METHOD_NOT_ALLOWED, "/405.html"),
            (StatusCode::LENGTH_REQUIRED, "/411.html"),
            (StatusCode::PAYLOAD_TOO_LARGE, "/413.html"),
            (StatusCode::INTERNAL_SERVER_ERROR, "/500.html"),
        ]),
        body_size_limit: 1024,
        routes: vec![Route {
            paths: vec!["/path1", "/path2"],
            accepted_http_methods: vec![http::Method::GET],
            http_redirections: HashMap::from([("/test1", "/path1"), ("/test2", "/path2")]),
            redirect_status_code: StatusCode::PERMANENT_REDIRECT,
            root_path: Some("src"),
            default_if_url_is_dir: "some default",
            default_if_request_is_dir: "some other default",
            cgi_def: HashMap::from([
                (".php", Cgi::PHP),
                (".py", Cgi::Python),
                (".js", Cgi::JavaScript),
                (".cpp", Cgi::Cpp),
            ]),
            list_directory: true,
            length_required: true,
        }],
    }]
}
