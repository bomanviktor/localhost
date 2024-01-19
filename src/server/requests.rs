use crate::log;
use crate::log::*;
use crate::server::{Request, Route, ServerConfig, StatusCode};
use crate::type_aliases::Bytes;

pub fn get_request(
    conf: &ServerConfig,
    request_parts: (String, Bytes),
) -> Result<Request<Bytes>, StatusCode> {
    let head = &request_parts.0;
    let body = request_parts.1;
    let version = version::get_version(head)?;
    let path = path::get_path(head);
    let method = super::get_method(head)?;

    // Constructing the request with parsed headers and body
    let mut request_builder = http::Request::builder()
        .method(method)
        .uri(path)
        .version(version);

    for header in headers::get_headers(head) {
        if let Some((key, value)) = headers::format_header(header) {
            request_builder =
                request_builder.header(key.to_ascii_lowercase(), value.to_ascii_lowercase());
        }
    }

    let body = if headers::is_chunked(request_builder.headers_ref()) {
        body::get_chunked_body(body, conf.body_size_limit)?
    } else {
        body::get_body(body, conf.body_size_limit)?
    };

    match request_builder.body(body) {
        Ok(request) => Ok(request),
        Err(request) => {
            log!(
                LogFileType::Server,
                format!("Error: Failed to get body {}", request)
            );
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub mod path {
    use super::*;
    use crate::server::utils::{get_line, get_split_index};
    use crate::type_aliases::Path;

    /// `path` gets the path from the `request`
    pub fn get_path(req: &str) -> &str {
        let line = get_line(req, 0);
        get_split_index(line, 1)
    }

    /// `path_exists` gets the `path` and the `index` of the `route` it was a part of if found.
    pub fn path_exists<'a>(
        requested_path: Path<'a>,
        routes: &[Route<'a>],
    ) -> Option<(usize, Path<'a>)> {
        // Check for _exact_ matches in path
        for (i, route) in routes.iter().enumerate() {
            if route.url_path == requested_path {
                return Some((i, route.url_path));
            }
            if route.settings.is_none() {
                continue;
            }
            if let Some(redirections) = route.settings.clone().unwrap().http_redirections {
                if redirections.contains(&requested_path) {
                    return Some((i, route.url_path));
                }
            }
        }

        let mut path_str = "";
        let mut index = usize::MAX;

        // Check for paths with matching roots
        for (i, route) in routes.iter().enumerate() {
            if !requested_path.starts_with(route.url_path) {
                continue;
            }

            // Sort the routes by length. More specified routes are prioritized
            // Example: "/foo" and "/foo/bar" both match "/foo/bar/baz". This will take the "/foo/bar" route.
            if path_str.is_empty() || route.url_path.len() > path_str.len() {
                path_str = route.url_path;
                index = i;
            }
        }

        if index == usize::MAX {
            None
        } else {
            Some((index, path_str))
        }
    }

    pub fn add_root_to_path(route: &Route, path: &str) -> String {
        if let Some(settings) = &route.settings {
            let root = settings.root_path.unwrap_or_default();
            format!(".{root}{path}")
        } else {
            format!(".{path}")
        }
    }
}

pub mod version {
    use crate::log;
    use crate::log::*;
    use http::{StatusCode, Version};

    pub fn get_version(req: &str) -> Result<Version, StatusCode> {
        let version_str = req
            .split_whitespace()
            .find(|s| s.contains("HTTP/"))
            .ok_or(StatusCode::BAD_REQUEST)?;

        match version_str {
            "HTTP/0.9" => Ok(Version::HTTP_09),
            "HTTP/1.0" => Ok(Version::HTTP_10),
            "HTTP/1.1" => Ok(Version::HTTP_11),
            "HTTP/2.0" => Ok(Version::HTTP_2),
            "HTTP/3.0" => Ok(Version::HTTP_3),
            _ => {
                log!(
                    LogFileType::Server,
                    format!("Error: Version not supported {}", version_str)
                );
                Err(StatusCode::HTTP_VERSION_NOT_SUPPORTED)
            }
        }
    }
}

pub mod headers {
    use http::header::TRANSFER_ENCODING;
    use http::HeaderMap;

    pub fn get_headers(req: &str) -> Vec<&str> {
        // Remove the body from the request
        let head = req
            .trim_end_matches('\n')
            .trim_end()
            .split("\r\n\r\n")
            .collect::<Vec<&str>>()[0];

        head.trim_end()
            .split("\r\n")
            .filter(|line| !line.contains("HTTP/"))
            .collect::<Vec<&str>>()
    }

    pub fn is_chunked(headers: Option<&HeaderMap>) -> bool {
        if headers.is_none() {
            return false;
        }

        if let Some(header) = headers.unwrap().get(TRANSFER_ENCODING) {
            header.to_str().unwrap() == "chunked"
        } else {
            false
        }
    }

    pub fn format_header(header: &str) -> Option<(&str, &str)> {
        let key_value = header
            .trim_end_matches('\0')
            .trim_end()
            .split(": ")
            .collect::<Vec<&str>>();

        if key_value.len() == 2 {
            Some((key_value[0], key_value[1]))
        } else {
            None
        }
    }
}

pub mod body {
    use crate::log;
    use crate::log::LogFileType;
    use crate::type_aliases::Bytes;
    use http::StatusCode;

    pub fn get_body(body: Bytes, limit: usize) -> Result<Bytes, StatusCode> {
        if body.len() <= limit {
            Ok(body)
        } else {
            Err(StatusCode::PAYLOAD_TOO_LARGE)
        }
    }

    pub(crate) fn get_chunked_body(body: Bytes, limit: usize) -> Result<Bytes, StatusCode> {
        let mut result_body = Vec::new();
        let mut remaining_data = &body[..];

        while !remaining_data.is_empty() {
            // Split at the first occurrence of CRLF
            if let Some((size_str, rest)) = split_once_str(remaining_data, b'\r', b'\n') {
                // Parse the chunk size
                let chunk_size =
                    match usize::from_str_radix(String::from_utf8_lossy(size_str).trim(), 16) {
                        Ok(size) => size,
                        Err(_) => {
                            log!(
                                LogFileType::Server,
                                "Error: Failed to parse chunk size".to_string()
                            );
                            return Err(StatusCode::BAD_REQUEST);
                        }
                    };

                // Check for the end of the chunked body
                if chunk_size == 0 {
                    break;
                }

                // Ensure there's enough data for the chunk
                if rest.len() < chunk_size + 2 {
                    log!(
                        LogFileType::Server,
                        "Error: Not enough data for chunk".to_string()
                    );
                    return Err(StatusCode::BAD_REQUEST);
                }

                // Extract the chunk data
                let (chunk_data, after_chunk) = rest.split_at(chunk_size);
                result_body.extend_from_slice(chunk_data);

                // Check body size limit
                if result_body.len() > limit {
                    log!(LogFileType::Server, "Error: Body too long".to_string());
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }

                // Prepare for the next iteration, skip past the chunk data and CRLF
                remaining_data = &after_chunk[2..];
            } else {
                log!(
                    LogFileType::Server,
                    "Error: Missing CRLF after chunk size".to_string()
                );
                return Err(StatusCode::BAD_REQUEST); // Missing CRLF after chunk size
            }
        }

        Ok(result_body)
    }

    // Function to split the byte slice at the first occurrence of delimiter1 and delimiter2
    fn split_once_str(data: &[u8], delimiter1: u8, delimiter2: u8) -> Option<(&[u8], &[u8])> {
        for (i, &byte) in data.iter().enumerate() {
            if byte == delimiter1 {
                let (chunk, rest) = data.split_at(i + 1);
                if let Some((_, rest)) = rest.split_first() {
                    let (next_chunk, _) = rest.split_at(
                        rest.iter()
                            .position(|&x| x == delimiter2)
                            .unwrap_or(rest.len()),
                    );
                    return Some((chunk, next_chunk));
                }
            }
        }
        None
    }
}

pub mod utils {
    /// `get_split_index` gets the `&str` at `index` after performing `split_whitespace`
    pub fn get_split_index(str: &str, index: usize) -> &str {
        let lines = str.split_whitespace().collect::<Vec<&str>>();
        if lines.is_empty() {
            ""
        } else if index >= lines.len() {
            lines[0]
        } else {
            lines[index]
        }
    }

    /// `get_line` gets the `&str` at `index` after performing `split('\n')`
    pub fn get_line(str: &str, index: usize) -> &str {
        let lines = str
            .trim_end_matches('\0')
            .split("\r\n")
            .collect::<Vec<&str>>();
        if lines.is_empty() {
            ""
        } else if index > lines.len() {
            lines[0]
        } else {
            lines[index]
        }
    }
}
