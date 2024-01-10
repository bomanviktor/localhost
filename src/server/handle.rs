use std::fs;

use crate::log;
use crate::log::*;
use crate::server::errors::error;
use crate::server::handle_method;
use crate::server::path::add_root_to_path;
use crate::server::redirections::redirect;
use crate::server::*;
use crate::server_config::route::Settings;
use http::header::CONTENT_TYPE;

pub fn handle_client(stream: &mut TcpStream, config: &ServerConfig) -> io::Result<()> {
    let mut buffer = [0; 10024];

    // Read from stream
    let bytes_read = stream.read(&mut buffer)?;

    // Parse the request
    let request_string = match String::from_utf8(buffer[..bytes_read].to_vec()) {
        Ok(request_str) => request_str,
        Err(e) => {
            log!(
                LogFileType::Server,
                format!("Error reading from buffer to string: {e}")
            );
            return Ok(());
        }
    };

    // Match the request with a route
    let request = match get_request(config, &request_string) {
        Ok(req) => req,
        Err(e) => {
            log!(LogFileType::Server, format!("Error: {}", e));
            return serve_response(stream, error(e, config));
        }
    };

    // Handle redirections
    let route = match get_route(&request, config) {
        Ok(route) => route,
        Err((code, path)) if code.is_redirection() => {
            return serve_response(stream, redirect(code, config, request.version(), path));
        }
        Err((code, _)) => {
            log!(LogFileType::Server, format!("Error: {}", &code));
            return serve_response(stream, error(code, config));
        }
    };

    // Use the routes' handler
    if let Some(handler) = route.handler {
        return match handler(&request, config) {
            Ok(response) => serve_response(stream, response),
            Err(code) => {
                log!(LogFileType::Server, format!("Error: {}", &code));
                serve_response(stream, error(code, config))
            }
        };
    }

    if let Some(settings) = &route.settings {
        let request_path = &request.uri().to_string();
        if is_cgi_request(request_path) {
            match execute_cgi_script(&request_string, config, settings) {
                Ok(resp) => {
                    stream.write_all(&resp).unwrap();
                    stream.flush().expect("could not flush");
                }
                Err(code) => {
                    log!(LogFileType::Server, format!("Error: {}", &code));
                    return serve_response(stream, error(code, config));
                }
            }
            return Ok(());
        }

        let path = &add_root_to_path(&route, request_path);
        if std::path::Path::new(&path).is_dir() {
            return serve_directory_contents(stream, path, settings);
        }
    }

    // Handle based on HTTP method
    match handle_method(&route, &request, config) {
        Ok(response) => serve_response(stream, response)?,
        Err(code) => {
            log!(LogFileType::Server, format!("Error: {}", &code));
            serve_response(stream, error(code, config))?
        }
    }
    Ok(())
}

pub fn serve_response(stream: &mut TcpStream, response: Response<Bytes>) -> io::Result<()> {
    unsafe {
        stream.write_all(&format_response(response))?;
    }
    stream.flush()
}

fn serve_directory_contents(
    stream: &mut TcpStream,
    path: &str,
    _settings: &Settings,
) -> io::Result<()> {
    let entries = fs::read_dir(path)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Directory not found"))?
        .map(|res| res.map(|e| e.file_name().into_string().unwrap_or_default()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    let body = format!(
        "<html><body><ul>{}</ul></body></html>",
        entries.into_iter().fold(String::new(), |acc, entry| acc
            + &format!("<li>{}</li>", entry))
    );

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .body(Bytes::from(body))
        .unwrap();

    serve_response(stream, response)
}
