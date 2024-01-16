use crate::log;
use crate::log::*;
use crate::server::path::add_root_to_path;
use crate::server::{get_route, Bytes, ServerConfig, StatusCode};
use crate::type_aliases::FileExtension;
use http::header::*;
use http::{HeaderMap, HeaderName, HeaderValue, Request, Response};
use serde::{self, de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::env;
use std::process::Command;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum Cgi {
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
    JavaScript,
    Julia,
    Kotlin,
    Lua,
    Nim,
    ObjectiveC,
    OCaml,
    Pascal,
    Perl,
    PHP,
    Python,
    R,
    Ruby,
    Rust,
    Scala,
    Shell,
    Swift,
    TypeScript,
    Zig,
}

impl FromStr for Cgi {
    type Err = serde::de::value::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Ada" => Ok(Cgi::Ada),
            "C" => Ok(Cgi::C),
            "CSharp" => Ok(Cgi::CSharp),
            "Cpp" => Ok(Cgi::Cpp),
            "D" => Ok(Cgi::D),
            "Erlang" => Ok(Cgi::Erlang),
            "Fortran" => Ok(Cgi::Fortran),
            "Go" => Ok(Cgi::Go),
            "Groovy" => Ok(Cgi::Groovy),
            "Haskell" => Ok(Cgi::Haskell),
            "Java" => Ok(Cgi::Java),
            "JavaScript" => Ok(Cgi::JavaScript),
            "Julia" => Ok(Cgi::Julia),
            "Kotlin" => Ok(Cgi::Kotlin),
            "Lua" => Ok(Cgi::Lua),
            "Nim" => Ok(Cgi::Nim),
            "ObjectiveC" => Ok(Cgi::ObjectiveC),
            "OCaml" => Ok(Cgi::OCaml),
            "Pascal" => Ok(Cgi::Pascal),
            "Perl" => Ok(Cgi::Perl),
            "PHP" => Ok(Cgi::PHP),
            "Python" => Ok(Cgi::Python),
            "R" => Ok(Cgi::R),
            "Ruby" => Ok(Cgi::Ruby),
            "Rust" => Ok(Cgi::Rust),
            "Scala" => Ok(Cgi::Scala),
            "Shell" => Ok(Cgi::Shell),
            "Swift" => Ok(Cgi::Swift),
            "TypeScript" => Ok(Cgi::TypeScript),
            "Zig" => Ok(Cgi::Zig),
            _ => Err(de::Error::custom(format!("unknown CGI type: {}", s))),
        }
    }
}

impl<'de> Deserialize<'de> for Cgi {
    fn deserialize<D>(deserializer: D) -> Result<Cgi, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Cgi::from_str(&s).map_err(serde::de::Error::custom)
    }
}

pub fn is_cgi_request(path: &str) -> bool {
    path.contains("/cgi/")
}

const STANDARD_HEADERS: [HeaderName; 1] = [TRANSFER_ENCODING];
pub fn execute_cgi_script(
    req: &Request<String>,
    config: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    let route = &get_route(req, config).unwrap();
    let settings = match &route.settings {
        Some(s) => s,
        None => return Err(StatusCode::BAD_REQUEST),
    };

    if settings.cgi_def.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let full_path = add_root_to_path(route, req.uri().path());
    let body = req.body().to_string();
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

    let file_extension_clone = file_extension.clone();

    // Check if the file extension is associated with a CGI script
    let cgi_map = settings
        .cgi_def
        .as_ref()
        .map(|map| {
            map.iter()
                .filter_map(|(ext, cgi_string)| {
                    if let Ok(cgi_enum) = Cgi::from_str(cgi_string) {
                        Some((ext.clone(), cgi_enum))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<String, Cgi>>()
        })
        .unwrap_or_default();

    let (command, arguments) = match cgi_map.get(&file_extension_clone) {
        Some(cgi_type) => match cgi_type {
            Cgi::Ada => ("ada", vec![full_path.clone(), body.clone()]),
            Cgi::C => ("./compiled/c_binary", vec![body.clone()]),
            Cgi::CSharp => ("dotnet", vec![full_path.clone(), body.clone()]),
            Cgi::Cpp => ("./compiled/cpp_binary", vec![body.clone()]),
            Cgi::D => ("dmd", vec![full_path.clone(), body.clone()]),
            Cgi::Erlang => ("escript", vec![full_path.clone(), body.clone()]),
            Cgi::Fortran => ("gfortran", vec![full_path.clone(), body.clone()]),
            Cgi::Go => (
                "go",
                vec!["run".to_string(), full_path.clone(), body.clone()],
            ),
            Cgi::Groovy => ("groovy", vec![full_path.clone(), body.clone()]),
            Cgi::Haskell => ("runhaskell", vec![full_path.clone(), body.clone()]),
            Cgi::Java => (
                "java",
                vec![
                    "-cp".to_string(),
                    "compiled".to_string(),
                    "Main".to_string(),
                ],
            ),
            Cgi::JavaScript => ("node", vec![full_path.clone(), body.clone()]),
            Cgi::Julia => ("julia", vec![full_path.clone(), body.clone()]),
            Cgi::Kotlin => (
                "kotlin",
                vec![
                    "-cp".to_string(),
                    "compiled".to_string(),
                    "MainKt".to_string(),
                    body.clone(),
                ],
            ),
            Cgi::Lua => ("lua", vec![full_path.clone(), body.clone()]),
            Cgi::Nim => (
                "nim",
                vec![
                    "c".to_string(),
                    "--run".to_string(),
                    full_path.clone(),
                    body.clone(),
                ],
            ),
            Cgi::ObjectiveC => ("./compiled/objc_binary", vec![body.clone()]),
            Cgi::OCaml => ("ocaml", vec![full_path.clone(), body.clone()]),
            Cgi::Pascal => ("fpc", vec![full_path.clone(), body.clone()]),
            Cgi::Perl => ("perl", vec![full_path.clone(), body.clone()]),
            Cgi::PHP => ("php", vec![full_path.clone(), body.clone()]),
            Cgi::Python => ("python", vec![full_path.clone(), body.clone()]),
            Cgi::R => ("Rscript", vec![full_path.clone(), body.clone()]),
            Cgi::Ruby => ("ruby", vec![full_path.clone(), body.clone()]),
            Cgi::Rust => ("./compiled/rust_binary", vec![body.clone()]),
            Cgi::Scala => ("scala", vec![full_path.clone(), body.clone()]),
            Cgi::Shell => ("sh", vec![full_path.clone(), body.clone()]),
            Cgi::Swift => ("swift", vec![full_path.clone(), body.clone()]),
            Cgi::TypeScript => ("ts-node", vec![full_path.clone(), body.clone()]),
            Cgi::Zig => ("./compiled/zig_binary", vec![body.clone()]),
        },
        None => {
            log!(
                LogFileType::Server,
                format!("Error: CGI not found {}", path)
            );
            return Err(StatusCode::NOT_FOUND);
        }
    };

    add_env_variables(req, config, file_extension);

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
        .header(HOST, config.host.clone())
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

fn add_env_variables(req: &Request<String>, config: &ServerConfig, file_extension: FileExtension) {
    add_http_variables(req.headers());
    if let Some(query) = req.uri().query() {
        env::set_var("QUERY_STRING", query);
    }

    env::set_var("REQUEST_METHOD", req.method().to_string());
    env::set_var("SERVER_NAME", config.host.clone());

    if let Some(port) = req.uri().port_u16() {
        env::set_var("SERVER_PORT", format!("{port}"));
    }

    env::set_var("SERVER_SOFTWARE", "Rust v1.74.0");

    let path = req
        .uri()
        .path()
        .split(&file_extension)
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
