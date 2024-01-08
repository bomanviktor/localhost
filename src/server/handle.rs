use std::fs;

use crate::log;
use crate::log::*;
use crate::server::errors::error;
use crate::server::handle_method;
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

    // Handle the request
    if let Some(handler) = route.handler {
        return match handler(&request, config) {
            Ok(response) => serve_response(stream, response),
            Err(code) => {
                log!(LogFileType::Server, format!("Error: {}", &code));
                serve_response(stream, error(code, config))
            }
        };
    } else if route.settings.as_ref().map_or(false, |s| s.list_directory) {
        let current_dir = std::env::current_dir()?;

        println!("Current directory: {:?}", current_dir);

        // Read the contents of the directory
        let entries = fs::read_dir(current_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Print the path
            if path.is_dir() {
                println!("Directory: {:?}", path);
            } else {
                println!("File: {:?}", path);
            }
        }

        let path = format!("./src{}/", request.uri().path());

        println!("Computed path for directory listing: {}", path);
        if std::path::Path::new(&path).is_dir() {
            println!("Confirmed directory. Proceeding to list contents.");
            return serve_directory_contents(stream, &path, &route.settings.unwrap());
        } else {
            println!("Path is not a directory.");
        }
        fs::read_dir(path.clone())
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Directory not found"))?;
        if std::path::Path::new(&path).is_dir() {
            println!("is dir");
            return serve_directory_contents(stream, &path, &route.settings.unwrap());
        }
    } else if route.settings.is_some() && is_cgi_request(&request.uri().to_string()) {
        match execute_cgi_script(&request_string, config, &route.settings.unwrap()) {
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
    println!("{response:?}");
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
    println!("Path: {}", path);
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
