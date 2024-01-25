use crate::log;
use crate::log::*;
use crate::server::errors::error;
use crate::server::handle_method;
use crate::server::path::add_root_to_path;
use crate::server::redirections::redirect;
use crate::server::safe::get;
use crate::server::*;
use serve::*;
use std::path::Path;

const KB: usize = 1024;
pub const BUFFER_SIZE: usize = KB;
pub fn handle_connection(stream: &mut TcpStream, config: &ServerConfig) -> io::Result<()> {
    let request_parts =
        unsafe { parse_http_request(stream) }.map_err(|_| io::Error::from_raw_os_error(35))?;
    let request = get_request(config, request_parts.clone())
        .map_err(|e| serve_response(stream, error(e, config)))
        .unwrap_or_else(|_| Default::default());

    // Get the route from the http::Request
    let route = match get_route(&request, config) {
        Ok(route) => route,

        // Handle the redirections
        Err((code, path)) if code.is_redirection() => {
            return serve_response(stream, redirect(code, config, request.version(), path));
        }

        // Handle the errors
        Err((code, _)) => {
            log!(LogFileType::Server, format!("Error: {}", &code));
            return serve_response(stream, error(code, config));
        }
    };

    // Use the associated handler for the route
    if let Some(handler) = route.handler {
        return match handler(&request, config) {
            Ok(response) => serve_response(stream, response),
            Err(code) => {
                log!(LogFileType::Server, format!("Error: {}", &code));
                serve_response(stream, error(code, config))
            }
        };
    }

    let path = &add_root_to_path(&route, request.uri().path());

    // Check if the path is a directory and a default file is specified
    if Path::new(&path).is_dir() && route.settings.is_some() {
        let settings = route.settings.as_ref().unwrap();

        // Serve the default file if enabled in config
        if let Some(default_file) = settings.default_if_url_is_dir {
            let default_path = &add_root_to_path(&route, default_file);
            let new_head =
                replace_path_in_request(request_parts.0, request.uri().path(), default_path);
            let request_parts = (new_head, request_parts.1);
            let request = match get_request(config, request_parts) {
                Ok(r) => r,
                Err(code) => {
                    log!(LogFileType::Server, code.to_string());
                    return serve_response(stream, error(code, config));
                }
            };

            return match get(&request, config) {
                Ok(resp) => serve_response(stream, resp),
                Err(e) => serve_response(stream, error(e, config)),
            };
        }

        // List directory setting is enabled. Default file is disabled.
        return if settings.list_directory {
            serve_directory_contents(stream, path)
        } else {
            serve_response(stream, error(StatusCode::NOT_FOUND, config))
        };
    }

    if is_cgi_request(path) {
        return match execute_cgi_script(&request, config) {
            Ok(resp) => serve_response(stream, resp),
            Err(code) => {
                log!(LogFileType::Server, format!("Error: {}", &code));
                serve_response(stream, error(code, config))
            }
        };
    }

    match handle_method(&route, &request, config) {
        Ok(response) => serve_response(stream, response),
        Err(code) => {
            log!(LogFileType::Server, format!("Error: {}", &code));
            serve_response(stream, error(code, config))
        }
    }
}

unsafe fn parse_http_request(stream: &mut TcpStream) -> Result<(String, Vec<u8>), u32> {
    let mut buffer = [0; BUFFER_SIZE];
    let mut head = String::new();
    let mut body = Vec::new();

    // Get the head and first bytes of the body
    loop {
        let bytes_read = stream.read(&mut buffer).map_err(|_| line!())?;

        if bytes_read == 0 {
            return Ok((head, body));
        }

        match String::from_utf8(buffer[..bytes_read].to_vec()) {
            Ok(chunk) => {
                if let Some(index) = chunk.find("\r\n\r\n") {
                    // Split head and body when finding the double CRLF (Carriage Return Line Feed)
                    head.push_str(&chunk[..index]);
                    body.extend(&buffer[index + 4..bytes_read]);
                    break;
                } else {
                    // If no double CRLF found, add the entire chunk to the head
                    head.push_str(&chunk);
                }
            }
            Err(_) => {
                let rest;
                unsafe {
                    rest = String::from_utf8_unchecked(buffer.to_vec());
                }
                let index = rest.find("\r\n\r\n").unwrap_or(0);
                head.push_str(rest.split_at(index).0);
                if index == 0 {
                    body.extend(&buffer[index..bytes_read]);
                } else {
                    body.extend(&buffer[index + 4..bytes_read]);
                }
                break;
            }
        }
        // Clear the buffer
    }

    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(b) => b,
            Err(_) => return Ok((head, body)),
        };
        body.extend(buffer);
        if bytes_read < BUFFER_SIZE {
            break;
        }
    }

    Ok((head, body))
}

fn replace_path_in_request(head: String, path: &str, default_path: &str) -> String {
    return if let Some(stripped_path) = path.strip_prefix('.') {
        head.replacen(stripped_path, &default_path[1..], 1)
    } else {
        head.replacen(path, &default_path[1..], 1)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_path_with_stripped_prefix() {
        let head = "GET /old_path HTTP/1.1\r\n".to_string();
        let path = "./old_path";
        let default_path = "/new_path";

        let result = replace_path_in_request(head, path, default_path);
        assert_eq!(result, "GET new_path HTTP/1.1\r\n");
    }

    #[test]
    fn replace_path_without_stripped_prefix() {
        let head = "POST /old_path HTTP/1.1\r\n".to_string();
        let path = "/old_path";
        let default_path = "/new_path";

        let result = replace_path_in_request(head, path, default_path);
        assert_eq!(result, "POST new_path HTTP/1.1\r\n");
    }

    #[test]
    fn replace_path_not_found() {
        let head = "PUT /another_path HTTP/1.1\r\n".to_string();
        let path = "/old_path";
        let default_path = "/new_path";

        let result = replace_path_in_request(head, path, default_path);
        assert_eq!(result, "PUT /another_path HTTP/1.1\r\n");
    }
}

mod serve {
    use crate::server::format_response;
    use crate::type_aliases::Bytes;
    use http::header::CONTENT_TYPE;
    use http::{Response, StatusCode};
    use mio::net::TcpStream;
    use std::io::Write;
    use std::path::Path;
    use std::{fs, io};

    pub fn serve_response(stream: &mut TcpStream, response: Response<Bytes>) -> io::Result<()> {
        let formatted_response = format_response(response.clone());
        let total_size = formatted_response.len();
        let mut written_size = 0;

        while written_size < total_size {
            match stream.write(&formatted_response[written_size..]) {
                Ok(0) => {
                    break; // No more data to write
                }
                Ok(n) => {
                    written_size += n;
                }
                Err(e) if e.kind() != io::ErrorKind::WouldBlock => {
                    return Err(e); // Error is not WouldBlock, return the error.
                }
                _ => {} // Event is now blocking, retry later.
            }
        }

        stream.flush()
    }

    pub fn serve_directory_contents(stream: &mut TcpStream, path: &str) -> io::Result<()> {
        // Ensure the path doesn't end with a slash
        let trimmed_path = path.trim_end_matches('/');

        let base_path = Path::new(trimmed_path);
        let entries = fs::read_dir(base_path)
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Directory not found"))?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;

        // Fold all entries into a single unordered list
        let body = format!(
            "<html><body><ul>{}</ul></body></html>",
            entries.into_iter().fold(String::new(), |acc, entry_path| {
                // Construct the relative path from the base path
                let relative_path = entry_path
                    .strip_prefix(base_path)
                    .unwrap_or(&entry_path)
                    .display()
                    .to_string();

                let entry_name = entry_path.file_name().unwrap_or_default().to_string_lossy();

                acc + &format!(
                    "<li><a href=\"/{}/{}\">{}</a></li>",
                    trimmed_path, relative_path, entry_name
                )
            })
        );

        let response = Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/html")
            .body(Bytes::from(body))
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Could not build response"))?;

        serve_response(stream, response)
    }
}
