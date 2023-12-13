use crate::client::{method, method_is_allowed, path, path_exists};
use crate::server_config::ServerConfig;
use http::{Response, StatusCode};
use std::io::{Read, Write};
use std::net::TcpStream;

pub fn handle_client(mut stream: TcpStream, config: &ServerConfig) {
    //println!("{config:#?}"); // Just printing the current config
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
    println!("Path {:?}", path);
    let method = method(&request);
    let mut route_index = 0;

    match path_exists(path, &config.routes) {
        Some((index, sanitized_path)) => {
            route_index = index; // Set the route index to pass on the correct information.
            if sanitized_path != path {
                path = sanitized_path; // Redirect the path
                status_code = StatusCode::PERMANENT_REDIRECT;
            }
        }
        None => status_code = StatusCode::NOT_FOUND,
    }

    if status_code == StatusCode::NOT_FOUND {
        // Error response here
        let response = Response::builder()
            .status(status_code)
            .header("Content-Type", "text/plain")
            .body("404 not found")
            .unwrap();

        let response = format!(
            "{:?} {}\nContent-Type: text/plain\n\n{}",
            response.version(),
            response.status(),
            response.body()
        );
        // Serve the response
        if let Err(error) = stream.write_all(response.as_bytes()) {
            eprintln!("Error writing response: {error}");
        }

        stream.flush().expect("could not flush");

        return;
    }

    // Do something based on method here. Can check for server configs etc.
    println!("{path:?}");

    if !method_is_allowed(method, &config.routes[route_index]) {
        status_code = StatusCode::METHOD_NOT_ALLOWED;
    }

    // Create the response
    let response = Response::builder()
        .status(status_code)
        .header("Content-Type", "text/plain")
        .body("Hello world")
        .unwrap();

    let response = format!(
        "{:?} {}\nContent-Type: text/plain\n\n{}",
        response.version(),
        response.status(),
        response.body()
    );

    // Serve the response
    if let Err(error) = stream.write_all(response.as_bytes()) {
        eprintln!("Error writing response: {error}");
    }

    stream.flush().expect("could not flush");
    //stream.write(TEMPORARY_RESPONSE.as_bytes()).expect("Something went terribly wrong.");
}
