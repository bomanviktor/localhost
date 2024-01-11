use std::fs;
use std::path::Path;

use crate::log;
use crate::log::*;
use crate::server::errors::error;
use crate::server::handle_method;
use crate::server::path::add_root_to_path;
use crate::server::redirections::redirect;
use crate::server::*;
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

    let path = &add_root_to_path(&route, request.uri());
    if std::path::Path::new(&path).is_dir() {
        return serve_directory_contents(stream, path);
    }

    if is_cgi_request(request.uri().path()) {
        return match execute_cgi_script(&request, config) {
            Ok(resp) => serve_response(stream, resp),
            Err(code) => {
                log!(LogFileType::Server, format!("Error: {}", &code));
                return serve_response(stream, error(code, config));
            }
        };
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

fn serve_directory_contents(stream: &mut TcpStream, path: &str) -> io::Result<()> {
    // Ensure the path doesn't end with a slash
    let trimmed_path = path.trim_end_matches('/');

    let base_path = Path::new(trimmed_path);
    let entries = fs::read_dir(base_path)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Directory not found"))?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    let body = format!(
        "<html><body><ul>{}</ul></body></html>",
        entries.into_iter().fold(String::new(), |acc, entry_path| {
            // Construct the relative path from the base path
            let relative_path = entry_path.strip_prefix(base_path)
                .unwrap_or(&entry_path)
                .display()
                .to_string();

            let entry_name = entry_path.file_name().unwrap_or_default().to_string_lossy();

            println!("entry_name: {}", entry_name);

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
        .unwrap();

    serve_response(stream, response)
}
