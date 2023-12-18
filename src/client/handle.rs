use crate::client::body::get_body;
use crate::client::cgi::{execute_cgi_script, is_cgi_request, CgiError};
use crate::client::errors::{client_error, server_error};
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

    let body = get_body(&request_str, config.body_size_limit).unwrap_or("".to_string());
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

    if is_cgi_request(path) {
        match execute_cgi_script(&request_str, config, route) {
            Ok(resp) => {
                stream.write_all(&resp).unwrap();
                stream.flush().expect("could not flush");
            }
            Err(e) => match e {
                CgiError::NotFound => {
                    serve_response(stream, client_error(StatusCode::NOT_FOUND, config, version))
                }
                CgiError::BodyTooLarge => serve_response(
                    stream,
                    client_error(StatusCode::PAYLOAD_TOO_LARGE, config, version),
                ),
                CgiError::ExecutionError => serve_response(
                    stream,
                    server_error(StatusCode::INTERNAL_SERVER_ERROR, config, version),
                ),
            },
        }
        return;
    }

    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .version(version)
        .header(HOST, config.host);

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
            let resp = resp.body(request.body()).unwrap();
            serve_response(stream, resp);
        }
        return;
    }

    let body = handle_method(route, path, method, None).unwrap_or_default();
    if body.is_empty() && !route.accepted_http_methods.contains(&Method::HEAD) {
        serve_response(stream, client_error(StatusCode::NOT_FOUND, config, version));
        return;
    }

    let resp = resp
        .header(CONTENT_TYPE, content_type(&request_str))
        .body(String::from_utf8(body).unwrap())
        .unwrap();

    serve_response(stream, resp)
}

fn serve_response<T: Display>(mut stream: TcpStream, response: Response<T>) {
    if let Err(error) = stream.write_all(&format(response)) {
        eprintln!("Error writing response: {error}");
    }
    stream.flush().expect("could not flush");
}

pub mod cgi {
    use crate::cgi::Cgi;
    use crate::client::body::get_body;
    use crate::client::cgi::CgiError::{BodyTooLarge, ExecutionError, NotFound};
    use crate::client::path::get_path;
    use crate::server_config::route::Route;
    use crate::server_config::ServerConfig;
    use crate::type_aliases::Bytes;
    use std::process::Command;

    pub enum CgiError {
        NotFound,
        BodyTooLarge,
        ExecutionError,
    }

    pub fn is_cgi_request(path: &str) -> bool {
        path.starts_with("/cgi/")
    }

    pub fn execute_cgi_script(
        request_str: &str,
        config: &ServerConfig,
        route: &Route,
    ) -> Result<Bytes, CgiError> {
        let cgi_path = get_path(request_str);
        let file_extension = cgi_path.split('.').rev().collect::<Vec<&str>>()[0].trim_end();
        let path = format!("{}{}", route.root_path.unwrap_or("src"), cgi_path);
        let body = match get_body(request_str, config.body_size_limit) {
            Some(b) => b.to_string(),
            None => return Err(BodyTooLarge),
        };

        // Check if the file extension is associated with a CGI script
        let (command, arguments) = match route.cgi_def.get(file_extension) {
            Some(cgi_type) => match cgi_type {
                Cgi::Ada => ("ada", vec![path, body]),
                Cgi::C => ("./compiled/c_binary", vec![body]), // Replace with actual compiled binary path
                Cgi::CSharp => ("dotnet", vec![path, body]), // Replace with actual compiled binary path
                Cgi::Cpp => ("./compiled/cpp_binary", vec![body]), // Replace with actual compiled binary path
                Cgi::D => ("dmd", vec![path, body]),
                Cgi::Erlang => ("escript", vec![path, body]),
                Cgi::Fortran => ("gfortran", vec![path, body]),
                Cgi::Go => ("go", vec!["run".to_string(), path, body]), // Replace with actual Go run command
                Cgi::Groovy => ("groovy", vec![path, body]),
                Cgi::Haskell => ("runhaskell", vec![path, body]),
                Cgi::Java => (
                    "java",
                    vec![
                        "-cp".to_string(),
                        "compiled".to_string(),
                        "Main".to_string(),
                    ],
                ), // Replace with actual compiled class path and main class
                Cgi::JavaScript => ("node", vec![path, body]),
                Cgi::Julia => ("julia", vec![path, body]),
                Cgi::Kotlin => (
                    "kotlin",
                    vec![
                        "-cp".to_string(),
                        "compiled".to_string(),
                        "MainKt".to_string(),
                        body,
                    ],
                ), // Replace with actual compiled class path and main class
                Cgi::Lua => ("lua", vec![path, body]),
                Cgi::Nim => (
                    "nim",
                    vec!["c".to_string(), "--run".to_string(), path, body],
                ),
                Cgi::ObjectiveC => ("./compiled/objc_binary", vec![body]), // Replace with actual compiled binary path
                Cgi::OCaml => ("ocaml", vec![path, body]),
                Cgi::Pascal => ("fpc", vec![path, body]),
                Cgi::Perl => ("perl", vec![path, body]),
                Cgi::PHP => ("php", vec![path, body]),
                Cgi::Python => ("python3", vec![path, body]),
                Cgi::R => ("Rscript", vec![path, body]),
                Cgi::Ruby => ("ruby", vec![path, body]),
                Cgi::Rust => (
                    "cargo",
                    vec![
                        "run".to_string(),
                        "--manifest-path".to_string(),
                        "Cargo.toml".to_string(),
                        path,
                        body,
                    ],
                ),
                Cgi::Scala => ("scala", vec![path, body]),
                Cgi::Shell => ("sh", vec![path, body]),
                Cgi::Swift => ("swift", vec![path, body]),
                Cgi::TypeScript => ("ts-node", vec![path, body]),
                Cgi::Zig => ("zig", vec!["run".to_string(), path, body]),
            },
            None => return Err(NotFound),
        };

        // Spawn a new process to execute the CGI script and capture its output
        match Command::new(command).args(arguments).output() {
            Ok(output) => Ok(output.stdout),
            Err(e) => {
                eprintln!("Error executing CGI script: {}", e);
                Err(ExecutionError)
            }
        }
    }
}
