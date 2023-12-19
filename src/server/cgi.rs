use crate::server::{Bytes, Route, ServerConfig, StatusCode};
use std::process::Command;

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

pub fn is_cgi_request(path: &str) -> bool {
    path.starts_with("/cgi/")
}

pub fn execute_cgi_script(
    request_str: &str,
    config: &ServerConfig,
    route: &Route,
) -> Result<Bytes, StatusCode> {
    let cgi_path = crate::server::path::get_path(request_str);
    let file_extension = cgi_path.split('.').rev().collect::<Vec<&str>>()[0].trim_end();
    let path = format!("{}{}", route.root_path.unwrap_or("src"), cgi_path);

    let body = match crate::server::body::get_body(request_str, config.body_size_limit) {
        Some(b) => b.to_string(),
        None => return Err(StatusCode::PAYLOAD_TOO_LARGE),
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
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Spawn a new process to execute the CGI script and capture its output
    match Command::new(command).args(arguments).output() {
        Ok(output) => Ok(output.stdout),
        Err(e) => {
            eprintln!("Error executing CGI script: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
