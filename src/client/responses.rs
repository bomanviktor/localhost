use crate::server_config::ServerConfig;
use http::StatusCode;
use std::path::Path;

pub fn response(code: StatusCode, path: &Path, config: &ServerConfig) -> Vec<u8> {
    let header = "Content-Type: text/plain";

    // Replace body with the actual contents of the path
    let body = match code {
        StatusCode::OK => path,
        StatusCode::PERMANENT_REDIRECT => Path::new("308 Permanent Redirect"),
        // These are the errors.
        _ => config.default_error_paths.get(&code).expect("get wrecked"),
    };

    let version = "HTTP/1.1"; // Change this to get the version from the request
    format!("{version} {code}\n{header}\n\n{body:?}")
        .as_bytes()
        .to_vec()
}
