use crate::log;
use crate::log::*;
use crate::server::path::add_root_to_path;
use crate::server::{get_route, Bytes, ServerConfig, StatusCode};
use crate::type_aliases::FileExtension;
use http::header::*;
use http::{HeaderMap, HeaderName, HeaderValue, Request, Response};
use std::env;
use std::process::Command;

#[derive(Clone, Debug)]
pub enum Cgi {
    /*
    Ada,
    C,
    CSharp,
    Cpp,
    D,
    Erlang,
    Fortran,
    Go,
    Groovy,
    Haskell,
    Java,
     */
    JavaScript,
    /*
    Julia,
    Kotlin,
    Lua,
    Nim,
    ObjectiveC,
    OCaml,
    Pascal,
    Perl,

     */
    PHP,
    Python,
    // R,
    Ruby,
    /*
    Rust,
    Scala,
    Shell,
    Swift,
    TypeScript,
    Zig,
     */
}

pub fn is_cgi_request(path: &str) -> bool {
    path.contains("/cgi/")
}

const STANDARD_HEADERS: [HeaderName; 1] = [TRANSFER_ENCODING];
pub fn execute_cgi_script(
    req: &Request<Bytes>,
    config: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    let route = match get_route(req, config) {
        Ok(route) => route,
        Err((status, _)) => return Err(status),
    };

    let settings = match &route.settings {
        Some(s) => s,
        None => return Err(StatusCode::BAD_REQUEST),
    };

    if settings.cgi_def.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let full_path = add_root_to_path(&route, req.uri().path());
    let body = match String::from_utf8(req.body().clone()) {
        Ok(b) => b,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    let extension = full_path.split('.').rev().collect::<Vec<&str>>()[0].trim_end();

    let mut file_extension = String::new();

    for ch in extension.chars() {
        if ch.is_alphanumeric() {
            file_extension.push(ch);
        } else {
            break;
        }
    }

    let path = full_path
        .split(&format!(".{file_extension}"))
        .collect::<Vec<&str>>()[0]
        .to_string();

    let path = format!("{path}.{file_extension}");
    add_env_variables(req, config, file_extension.as_str());

    // Check if the file extension is associated with a CGI script
    let (command, arguments) = match settings
        .cgi_def
        .clone()
        .unwrap()
        .get(file_extension.as_str())
    {
        Some(cgi_type) => match cgi_type {
            /*
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

             */
            Cgi::JavaScript => ("node", vec![path, body]),
            /*
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
             */
            Cgi::PHP => ("php", vec![path, body]),
            Cgi::Python => ("python3", vec![path, body]),
            //Cgi::R => ("Rscript", vec![path, body]),
            Cgi::Ruby => ("ruby", vec![path, body]),
            /*
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
             */
        },

        None => {
            log!(
                LogFileType::Server,
                format!("Error: CGI not found {}", path)
            );
            return Err(StatusCode::NOT_FOUND);
        }
    };

    // Spawn a new process to execute the CGI script and capture its output
    let body = match Command::new(command).args(arguments).output() {
        Ok(output) => output.stdout,
        Err(e) => {
            log!(
                LogFileType::Server,
                format!("Error executing CGI script: {}", e)
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let mut resp = Response::builder()
        .version(req.version())
        .header(HOST, config.host)
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .header(CONTENT_LENGTH, body.len());

    for (key, value) in req.headers() {
        if STANDARD_HEADERS.contains(key) {
            resp = resp.header(key, value);
        }
    }

    let response = resp
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

fn add_env_variables(req: &Request<Bytes>, config: &ServerConfig, file_extension: FileExtension) {
    add_http_variables(req.headers());
    if let Some(query) = req.uri().query() {
        env::set_var("QUERY_STRING", query);
    }

    env::set_var("REQUEST_METHOD", req.method().to_string());
    env::set_var("SERVER_NAME", config.host);

    if let Some(port) = req.uri().port_u16() {
        env::set_var("SERVER_PORT", format!("{port}"));
    }

    env::set_var("SERVER_SOFTWARE", "Rust v1.74.0");

    let path = req
        .uri()
        .path()
        .split(file_extension)
        .collect::<Vec<&str>>();

    // localhost:8080/cgi/python.py/path/to/file -> PATH_INFO: /path/to/file
    if contains_path_info(path.clone()) {
        env::set_var("PATH_INFO", path[1]);
    }
}
fn add_http_variables(headers: &HeaderMap<HeaderValue>) {
    for (key, v) in headers {
        let value = v.to_str().unwrap_or_default();
        if value.is_empty() {
            continue;
        }
        match *key {
            ACCEPT => env::set_var("HTTP_ACCEPT", value),
            CONTENT_LENGTH => env::set_var("CONTENT_LENGTH", value),
            CONTENT_TYPE => env::set_var("CONTENT_TYPE", value),
            ACCEPT_CHARSET => env::set_var("HTTP_ACCEPT_CHARSET", value),
            ACCEPT_ENCODING => env::set_var("HTTP_ACCEPT_ENCODING", value),
            ACCEPT_LANGUAGE => env::set_var("HTTP_ACCEPT_LANGUAGE", value),
            FORWARDED => env::set_var("HTTP_FORWARDED", value),
            HOST => env::set_var("HTTP_HOST", value),
            PROXY_AUTHORIZATION => env::set_var("HTTP_PROXY_AUTHORIZATION", value),
            USER_AGENT => env::set_var("HTTP_USER_AGENT", value),
            COOKIE => env::set_var("COOKIE", value),
            _ => {}
        }
    }
}

fn contains_path_info(path: Vec<&str>) -> bool {
    path.len() == 2
}
