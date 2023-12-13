use crate::client::{method, method_is_allowed, path, path_exists, Response};
use crate::server_config::ServerConfig;
use http::StatusCode;
use std::io::{Read, Write};
use std::net::TcpStream;

pub fn handle_client(mut stream: TcpStream, config: &ServerConfig) {
    let mut buffer = [0; 1024];

    // Attempt to read the stream into the buffer
    if let Err(error) = stream.read(&mut buffer) {
        eprintln!("Error reading from stream: {}", error);
        return;
    }

    // Attempt to convert the buffer to a String
    let request = match String::from_utf8(buffer.to_vec()) {
        Ok(request_str) => request_str,
        Err(error) => {
            eprintln!("Error converting buffer to String: {}", error);
            return;
        }
    };

    let mut status_code = StatusCode::OK;
    let mut path = path(&request);
    let method = method(&request);
    let mut route = &config.routes[0];

    match path_exists(path, &config.routes) {
        Some((i, sanitized_path)) => {
            route = &config.routes[i]; // Set the route index to pass on the correct information.
            if sanitized_path != path {
                path = sanitized_path; // Redirect the path
                status_code = StatusCode::PERMANENT_REDIRECT;
            }
        }
        None => status_code = StatusCode::NOT_FOUND,
    }

    if !method_is_allowed(&method, route) {
        status_code = StatusCode::METHOD_NOT_ALLOWED;
    }

    let response = Response::new(status_code, method, path, config);

    // Serve the response
    if let Err(error) = stream.write_all(&response.format()) {
        eprintln!("Error writing response: {error}");
    }

    stream.flush().expect("could not flush");
}
