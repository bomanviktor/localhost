use crate::client::body::get_body;
use crate::client::errors::client_error;
use crate::client::headers::{format_header, get_content_length, get_content_type, get_headers};
use crate::client::method::{get_method, handle_method, method_is_allowed};
use crate::client::path::{get_path, path_exists};
use crate::client::redirections::{is_redirect, redirect};
use crate::client::utils::to_bytes;
use crate::client::version::get_version;
use crate::client::{content_type, format};
use crate::server_config::ServerConfig;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE, HOST};
use http::{Method, Request, Response, StatusCode};
use std::fmt::Display;
use std::io::{Read, Write};
use std::net::TcpStream;

const STATE_CHANGING_METHODS: [Method; 4] =
    [Method::PUT, Method::POST, Method::PATCH, Method::DELETE];

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
    let path = get_path(&request_str);
    let method = match get_method(&request_str) {
        Ok(method) => method,
        Err(_) => {
            serve_response(
                stream,
                client_error(StatusCode::METHOD_NOT_ALLOWED, config, version),
            );
            return;
        }
    };

    let mut request = Request::builder()
        .method(&method)
        .uri(path)
        .version(version);

    for header in get_headers(&request_str) {
        if let Some((key, value)) = format_header(header) {
            request = request.header(key, value);
        }
    }

    let body = get_body(&request_str, config.body_size_limit).unwrap_or("");
    let request = request.body(body).unwrap();

    // Get the route assigned to the path
    let route;
    if let Some((i, sanitized_path)) = path_exists(path, &config.routes) {
        route = &config.routes[i]; // Set the route index to pass on the correct information.
        if is_redirect(path, sanitized_path) {
            serve_response(
                stream,
                redirect(route.redirect_status_code, config, version, sanitized_path),
            );
            return;
        }
    } else {
        serve_response(stream, client_error(StatusCode::NOT_FOUND, config, version));
        return;
    }

    if !method_is_allowed(&method, route) {
        serve_response(
            stream,
            client_error(StatusCode::METHOD_NOT_ALLOWED, config, version),
        );
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
            serve_response(
                stream,
                client_error(StatusCode::BAD_REQUEST, config, version),
            ); // Respond with bad request if state changing method but no content type
            return;
        }

        let mut content_length = false;
        if let Some(length) = get_content_length(&request_str) {
            resp = resp.header(CONTENT_LENGTH, length);
            content_length = true;
        }

        // Requires a length, has no length, and is a
        if route.length_required && !content_length {
            serve_response(
                stream,
                client_error(StatusCode::LENGTH_REQUIRED, config, version),
            );
            return;
        }

        // Get the body of the response
        if request.body().len() > config.body_size_limit {
            serve_response(
                stream,
                client_error(StatusCode::PAYLOAD_TOO_LARGE, config, version),
            );
        } else {
            handle_method(route, path, method, Some(to_bytes(request.body())));
            let resp = resp.body(*request.body()).unwrap();
            serve_response(stream, resp);
        }
        return;
    }

    let body = handle_method(route, path, method, None);
    let resp = resp
        .header(CONTENT_TYPE, content_type(&request_str))
        .body(String::from_utf8(body.unwrap_or_default()).unwrap())
        .unwrap();

    serve_response(stream, resp)
}

fn serve_response<T: Display>(mut stream: TcpStream, response: Response<T>) {
    if let Err(error) = stream.write_all(&format(response)) {
        eprintln!("Error writing response: {error}");
    }
    stream.flush().expect("could not flush");
}
