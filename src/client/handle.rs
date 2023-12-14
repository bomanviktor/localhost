use crate::client::headers::{get_content_length, get_content_type};
use crate::client::method::{method, method_is_allowed};
use crate::client::path::{path, path_exists};
use crate::client::Response;
use crate::server_config::ServerConfig;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE, HOST, LOCATION};
use http::{HeaderMap, Method, StatusCode};
use std::io::{Read, Write};
use std::net::TcpStream;

const STATE_CHANGING_METHODS: [Method; 3] = [Method::PUT, Method::POST, Method::PATCH];

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

    let mut resp_header = HeaderMap::new();
    resp_header.insert(HOST, config.host.parse().unwrap());

    let mut status_code = StatusCode::OK;
    let mut path = path(&request);
    let mut method = method(&request);
    let mut route = &config.routes[0];

    match path_exists(path, &config.routes) {
        Some((i, sanitized_path)) => {
            route = &config.routes[i]; // Set the route index to pass on the correct information.
            if sanitized_path != path {
                resp_header.insert(LOCATION, sanitized_path.parse().unwrap()); // Add the redirected path to the header
                path = sanitized_path; // Redirect the path
                status_code = StatusCode::PERMANENT_REDIRECT; // TODO: Make it possible to choose appropriate status code
            }
        }
        None => status_code = StatusCode::NOT_FOUND,
    }

    if !method_is_allowed(&method, route) {
        status_code = StatusCode::METHOD_NOT_ALLOWED;
    }

    // Status Code is an error. Change the method to `GET` to send the error page
    if is_error(&status_code) {
        method = Method::GET;
    }

    // Content-Length
    if let Some(length) = get_content_length(&request) {
        resp_header.insert(CONTENT_LENGTH, length.parse().unwrap());
    }

    if route.length_required                              // Content-Length is required
        && resp_header.get(CONTENT_LENGTH).is_none() // Content-Length is absent
        && STATE_CHANGING_METHODS.contains(&method)
    // The method changes the server state
    {
        status_code = StatusCode::LENGTH_REQUIRED;
    }

    // Content-Type for server-changing requests
    if STATE_CHANGING_METHODS.contains(&method) {
        if let Some(content_type) = get_content_type(&request) {
            resp_header.insert(CONTENT_TYPE, content_type.parse().unwrap());
        }
    }

    let response = Response::new(
        resp_header,
        status_code,
        &method,
        path,
        config,
        if STATE_CHANGING_METHODS.contains(&method) {
            println!("Get the bytes here!");
            Some(vec![1, 2, 3])
        } else {
            None
        },
    );

    // Serve the response
    if let Err(error) = stream.write_all(&response.format()) {
        eprintln!("Error writing response: {error}");
    }

    stream.flush().expect("could not flush");
}

fn is_error(code: &StatusCode) -> bool {
    code.is_client_error() || code.is_server_error()
}
