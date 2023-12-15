use crate::client::errors::client_errors::{
    bad_request, length_required, method_not_allowed, not_found, payload_too_large,
};
use crate::client::headers::{get_content_length, get_content_type};
use crate::client::method::{handle_method, method, method_is_allowed};
use crate::client::path::{path, path_exists};
use crate::client::redirections::temporary_redirect;
use crate::client::utils::to_bytes;
use crate::client::{content_type, format};
use crate::server_config::ServerConfig;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE, HOST};
use http::{Method, Response, StatusCode, Version};
use std::fmt::Display;
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
    let request_str = match String::from_utf8(buffer.to_vec()) {
        Ok(request_str) => request_str,
        Err(error) => {
            eprintln!("Error converting buffer to String: {}", error);
            return;
        }
    };

    let version = get_version(&request_str);
    let path = path(&request_str);
    let method = match method(&request_str) {
        Ok(method) => method,
        Err(_) => {
            serve_response(stream, method_not_allowed(config, version));
            return;
        }
    };

    #[allow(unused_assignments)]
    let mut route = &config.routes[0];
    if let Some((i, sanitized_path)) = path_exists(path, &config.routes) {
        route = &config.routes[i]; // Set the route index to pass on the correct information.
        if sanitized_path != path {
            // TODO: Implementation for route.default_redirect_method
            serve_response(stream, temporary_redirect(config.host, path, version));
            return;
        }
    } else {
        serve_response(stream, not_found(config, version));
        return;
    }

    if !method_is_allowed(&method, route) {
        serve_response(stream, method_not_allowed(config, version));
        return;
    }

    let mut resp = Response::builder()
        .version(version)
        .header(HOST, config.host)
        .status(StatusCode::OK);

    // State changing http methods
    if STATE_CHANGING_METHODS.contains(&method) {
        if let Some(content_type) = get_content_type(&request_str) {
            resp = resp.header(CONTENT_TYPE, content_type);
        } else {
            serve_response(stream, bad_request(config, version)); // Respond with bad request if state changing method but no content type
            return;
        }

        let mut content_length = false;
        if let Some(length) = get_content_length(&request_str) {
            resp = resp.header(CONTENT_LENGTH, length);
            content_length = true;
        }

        // Requires a length, has no length, and is a
        if route.length_required && !content_length {
            serve_response(stream, length_required(config, version));
            return;
        }

        if let Some(resp_body) = get_body(&request_str, config.body_size_limit) {
            handle_method(path, method, Some(to_bytes(resp_body))); // do stuff to server
            let resp = resp.body(resp_body).unwrap();
            serve_response(stream, resp);
        } else {
            serve_response(stream, payload_too_large(config, version));
        }
        return;
    }

    let body = handle_method(path, method, None);
    let resp = resp
        .version(version)
        .header(CONTENT_TYPE, content_type(&request_str))
        .status(StatusCode::OK)
        .body(String::from_utf8(body.unwrap_or_default()).unwrap())
        .unwrap();

    serve_response(stream, resp)
}

fn get_version(req: &str) -> Version {
    let version_str = req
        .split_whitespace()
        .find(|s| s.contains("HTTP/"))
        .unwrap_or("HTTP/1.1");

    match version_str {
        "HTTP/0.9" => Version::HTTP_09,
        "HTTP/1.0" => Version::HTTP_10,
        "HTTP/1.1" => Version::HTTP_11,
        "HTTP/2.0" => Version::HTTP_2,
        "HTTP/3.0" => Version::HTTP_3,
        _ => Version::HTTP_11,
    }
}

fn get_body(req: &str, limit: usize) -> Option<&str> {
    let binding = req
        .trim_end_matches('\0')
        .split("\n\n")
        .collect::<Vec<&str>>();

    let body = *binding.last().unwrap_or(&"");

    if body.len() <= limit {
        Some(body)
    } else {
        None
    }
}

fn serve_response<T: Display>(mut stream: TcpStream, response: Response<T>) {
    if let Err(error) = stream.write_all(&format(response)) {
        eprintln!("Error writing response: {error}");
    }
    stream.flush().expect("could not flush");
}
