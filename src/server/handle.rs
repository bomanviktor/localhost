use std::path::Path;

use crate::log;
use crate::log::*;
use crate::server::errors::error;
use crate::server::handle_method;
use crate::server::path::add_root_to_path;
use crate::server::redirections::redirect;
use crate::server::safe::get;
use crate::server::*;
use serve::*;

pub fn handle_connection(stream: &mut TcpStream, config: &ServerConfig) -> io::Result<()> {
    let mut request_string = String::new();
    let mut buffer = [0; 256];

    // Read into buffer
    while let Ok(bytes_read) = stream.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        match String::from_utf8(buffer[..bytes_read].to_vec()) {
            Ok(str) => request_string.push_str(&str),
            Err(_) => {
                unsafe {
                    // Insert the buffer into the request string
                    request_string
                        .push_str(&String::from_utf8_unchecked(buffer[..bytes_read].to_vec()));
                }
            }
        }
        // Clear the buffer
        buffer = [0; 256];
    }

    // Get the http::Request<String> type from the request string
    let request = match get_request(config, &request_string) {
        Ok(req) => req,
        Err(e) => {
            log!(LogFileType::Server, format!("Error: {}", e));
            return serve_response(stream, error(e, config));
        }
    };

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
            let request_string =
                replace_path_in_request(&request_string, request.uri().path(), default_path);

            let request = match get_request(config, &request_string) {
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

fn replace_path_in_request(req_string: &str, path: &str, default_path: &str) -> String {
    return if let Some(stripped_path) = path.strip_prefix('.') {
        req_string.replacen(stripped_path, &default_path[1..], 1)
    } else {
        req_string.replacen(path, &default_path[1..], 1)
    };
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
        let formatted_response = unsafe { format_response(response.clone()) };
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
