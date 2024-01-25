use crate::server::{Bytes, Response, ServerConfig, StatusCode, BUFFER_SIZE};
use http::header::TRANSFER_ENCODING;
use http::Version;
use std::fs;

pub fn format_response(response: Response<Bytes>) -> Bytes {
    // Split up the response into head and parts
    let (head, body) = response.into_parts();
    let mut resp = Bytes::from(format!("{:?} {}\r\n", head.version, head.status));

    // Get all headers into the response
    for (key, value) in head.headers.iter() {
        let key = key.to_string();
        let value = value.to_str().unwrap_or_default();
        let header = Bytes::from(format!("{key}: {value}\r\n"));
        resp.extend(header);
    }

    resp.extend("\r\n".as_bytes()); // Add the extra CRLF before the response body

    if body.is_empty() {
        return resp;
    }

    let chunk_size = if body.len() < BUFFER_SIZE {
        body.len()
    } else {
        BUFFER_SIZE
    };
    if is_chunked(head) {
        for chunk in body.chunks(chunk_size) {
            resp.extend(format!("{:X}\r\n", chunk.len()).as_bytes());
            resp.extend(chunk);
            resp.extend("\r\n".as_bytes());
        }
        resp.extend("0\r\n\r\n".as_bytes()); // End of chunks
    } else {
        resp.extend(body); // No chunks
    }

    resp
}
fn is_chunked(head: http::response::Parts) -> bool {
    head.headers
        .get_all(TRANSFER_ENCODING)
        .iter()
        .any(|value| value.to_str().unwrap_or_default().to_uppercase() == "CHUNKED")
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
    use http::header::HOST;

    #[allow(dead_code)]
    pub fn informational(
        status: StatusCode,
        config: &ServerConfig,
        version: Version,
    ) -> Response<Bytes> {
        http::Response::builder()
            .version(version)
            .header(HOST, config.host)
            .status(status)
            .body(vec![])
            .unwrap()
    }
}

pub mod redirections {
    use super::*;
    use crate::type_aliases::Path;
    use http::header::{HOST, LOCATION};

    pub fn redirect(
        status: StatusCode,
        config: &ServerConfig,
        version: Version,
        path: String,
    ) -> Response<Bytes> {
        http::Response::builder()
            .version(version)
            .header(HOST, config.host)
            .header(LOCATION, path)
            .status(status)
            .body(vec![])
            .unwrap()
    }

    pub fn is_redirect(path: &str, redirections: &Option<Vec<Path>>) -> bool {
        if redirections.is_none() {
            return false;
        }

        redirections.clone().unwrap().contains(&path)
    }
}

pub mod errors {
    use super::*;
    use http::header::{CONTENT_LENGTH, HOST};

    pub fn error(code: StatusCode, config: &ServerConfig) -> Response<Bytes> {
        let error_body = check_errors(code, config).unwrap_or(Bytes::from(format!("{code}")));
        Response::builder()
            .header(HOST, config.host)
            .header(CONTENT_LENGTH, error_body.len())
            .status(code)
            .body(error_body)
            .unwrap()
    }

    fn generate_error_html(code: u16, name: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{code} {name}</title>
    <style>
        body {{
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
        }}

        h1 {{
            text-align: center;
            font-size: xxx-large;
            font-family: sans-serif;
        }}
    </style>
</head>
<body>
<h1>{code} | {name}</h1>
</body>
</html>"#,
        )
    }

    fn check_errors(code: StatusCode, config: &ServerConfig) -> std::io::Result<Bytes> {
        if let Some(custom_error_path) = config.custom_error_path {
            fs::read(format!(".{custom_error_path}/{}.html", code.as_u16()))
        } else {
            let name = code.canonical_reason().unwrap_or_default();
            let code = code.as_u16();
            Ok(Bytes::from(generate_error_html(code, name)))
        }
    }
}
