use crate::server::utils::to_bytes;
use crate::server::{Bytes, Response, ServerConfig, StatusCode};
use http::Version;
use std::fs;
use std::str::from_utf8_unchecked;

/// # Safety
///
/// This function will call the unsafe method `from_utf8_unchecked`
pub unsafe fn format_response(response: Response<Bytes>) -> Bytes {
    // Split up the response into head and parts
    let (head, body) = response.into_parts();
    let mut resp = format!("{:?} {}\r\n", head.version, head.status);

    // Get all headers into the response
    for (key, value) in head.headers.iter() {
        let key = key.to_string();
        let value = value.to_str().unwrap_or_default();
        let header = format!("{key}: {value}\r\n");
        resp.push_str(&header);
    }

    if !body.is_empty() {
        unsafe {
            resp.push_str(&format!("\r\n{}", from_utf8_unchecked(&body)));
        }
    } else {
        resp.push_str("\r\n");
    }

    to_bytes(&resp)
}

pub fn content_type(path: &str) -> String {
    let file_extension = path.split('.').rev().collect::<Vec<&str>>()[0];
    // "/test.html" -> "html"
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
        _ => "text/plain",
    }
    .to_string()
}

pub mod informational {
    use super::*;
    use http::header::{HOST, SERVER};

    #[allow(dead_code)]
    fn informational(
        status: StatusCode,
        config: &ServerConfig,
        version: Version,
    ) -> Response<Bytes> {
        http::Response::builder()
            .version(version)
            .header(HOST, config.host) // Replace with your actual header
            .header(SERVER, "grit:lab-localhost/1.0") // Replace with your actual server name and version
            .status(status)
            .body(vec![])
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
        path: String,
    ) -> Response<Bytes> {
        http::Response::builder()
            .version(version)
            .header(HOST, config.host)
            .header(SERVER, "grit:lab-localhost/1.0")
            .header(LOCATION, path)
            .status(status)
            .body(vec![])
            .unwrap()
    }

    pub fn is_redirect(path: &str, other_path: &str) -> bool {
        path != other_path
    }
}

pub mod errors {
    use super::*;
    use http::header::{CONTENT_LENGTH, HOST, SERVER};

    pub fn error(code: StatusCode, config: &ServerConfig) -> Response<Bytes> {
        let error_body = match check_errors(code, config) {
            Ok(b) => b,
            Err(_) => to_bytes(&format!("{code}")),
        };

        Response::builder()
            .header(HOST, config.host)
            .header(SERVER, "grit:lab-localhost/1.0")
            .header(CONTENT_LENGTH, error_body.len())
            .status(code)
            .body(error_body)
            .unwrap()
    }

    fn check_errors(code: StatusCode, config: &ServerConfig) -> std::io::Result<Bytes> {
        let error_path = config.default_error_path.unwrap_or("src");
        fs::read(format!("{error_path}/{}.html", code.as_u16()))
    }
}
