use crate::client::utils::to_bytes;
use crate::server_config::ServerConfig;
use crate::type_aliases::Bytes;
use http::{Response, StatusCode, Version};
use std::fmt::Display;
use std::fs;

pub fn format<T: Display>(response: Response<T>) -> Bytes {
    let version = response.version();
    let binding = response.status();
    let status = binding.as_str();
    let mut resp = format!("{version:?} {status}\n");

    // Get all headers into the response
    for (name, value) in response.headers() {
        let name = name.to_string();
        let value = value.to_str().unwrap();
        let header = format!("{name}: {value}\n");
        resp.push_str(&header);
    }

    let body = response.body().to_string();
    if !body.is_empty() {
        resp.push_str(&format!("\n{body}"));
    }

    to_bytes(&resp)
}

pub fn content_type(path: &str) -> String {
    let file_extension = path.split('.').rev().collect::<Vec<&str>>()[0];
    // "/test.html" -> "html"

    format!(
        "Content-Type: {}",
        match file_extension {
            // Text
            "html" => "text/html",
            "css" => "text/css",
            "js" => "text/javascript",
            "txt" => "text/plain",
            "xml" => "text/xml",
            // Message
            "http" => "message/http",
            // Image
            "jpeg" | "jpg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            "svg" => "image/svg+xml",
            // Audio
            "aac" => "audio/aac",
            "eac3" => "audio/eac3",
            "mp3" => "audio/mpeg",
            "ogg" => "audio/ogg",
            // Video
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "ogv" => "video/ogg",
            // Application
            "json" => "application/json",
            "pdf" => "application/pdf",
            "zip" => "application/zip",
            "tar" => "application/x-tar",
            "gz" => "application/gzip",
            "exe" => "application/octet-stream",
            "msi" => "application/octet-stream",
            "woff" => "application/font-woff",
            "woff2" => "application/font-woff2",
            "ttf" => "application/font-sfnt",
            "otf" => "application/font-sfnt",
            // Default to HTML for unknown types
            _ => "text/html",
        }
    )
}

pub mod informational {
    use super::*;
    use http::header::{HOST, SERVER};

    #[allow(dead_code)]
    fn informational(
        status: StatusCode,
        config: &ServerConfig,
        version: Version,
    ) -> Response<String> {
        http::Response::builder()
            .version(version)
            .header(HOST, config.host) // Replace with your actual header
            .header(SERVER, "grit:lab-localhost/1.0") // Replace with your actual server name and version
            .status(status)
            .body("".to_string())
            .unwrap()
    }
}

pub mod redirections {
    use super::*;
    use http::header::{HOST, LOCATION, SERVER};

    pub fn redirect(
        status: StatusCode,
        config: &ServerConfig,
        version: Version,
        path: &str,
    ) -> Response<String> {
        http::Response::builder()
            .version(version)
            .header(HOST, config.host)
            .header(SERVER, "grit:lab-localhost/1.0")
            .header(LOCATION, path)
            .status(status)
            .body("".to_string())
            .unwrap()
    }

    pub fn is_redirect(path: &str, other_path: &str) -> bool {
        path != other_path
    }
}

pub mod errors {
    use super::*;
    use crate::client::utils::to_bytes;
    use crate::server_config::ServerConfig;
    use http::header::{HOST, SERVER};

    fn base_response(
        code: StatusCode,
        config: &ServerConfig,
        version: Version,
    ) -> Response<String> {
        let error_body = check_errors(code, config).unwrap_or(to_bytes("400"));

        Response::builder()
            .version(version)
            .header(HOST, config.host)
            .header(SERVER, "grit:lab-localhost/1.0")
            .status(code)
            .body(String::from_utf8(error_body).unwrap())
            .unwrap()
    }

    fn check_errors(code: StatusCode, config: &ServerConfig) -> std::io::Result<Bytes> {
        let error_path = config
            .default_error_paths
            .get(&code)
            .unwrap_or(&"/400.html");
        fs::read(format!("src/default_errors{error_path}"))
    }

    /// Create a response for all 4xx errors
    pub fn client_error(
        code: StatusCode,
        config: &ServerConfig,
        version: Version,
    ) -> Response<String> {
        base_response(code, config, version)
    }

    /// Create a response for all 5xx errors
    pub fn server_error(
        code: StatusCode,
        config: &ServerConfig,
        version: Version,
    ) -> Response<String> {
        base_response(code, config, version)
    }
}
