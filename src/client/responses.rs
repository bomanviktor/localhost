use crate::server_config::ServerConfig;
use http::{Method, StatusCode};
use std::fs;

pub struct Response<'a> {
    code: StatusCode,
    method: Method,
    path: &'a str,
    config: &'a ServerConfig<'a>,
}

impl<'a> Response<'a> {
    pub fn new(
        code: StatusCode,
        method: Method,
        path: &'a str,
        config: &'a ServerConfig<'a>,
    ) -> Self {
        Self {
            code,
            method,
            path,
            config,
        }
    }
    pub fn format(&self) -> Vec<u8> {
        let header = "Content-Type: text/html";
        let version = "HTTP/1.1"; // Change this to get the version from the request

        let body = match self.code {
            StatusCode::OK => match self.method {
                Method::GET => fs::read(self.path),
                Method::POST => fs::read(self.path),
                _ => fs::read(self.path),
            },
            StatusCode::PERMANENT_REDIRECT => fs::read(self.path),
            // These are the errors.
            _ => {
                let error_path = self
                    .config
                    .default_error_paths
                    .get(&self.code)
                    .expect("get wrecked");
                println!("{}", error_path);
                fs::read(format!("src/default_errors{error_path}"))
            }
        }
        .unwrap_or_default(); // Change this to 500 Internal server Error

        let body = String::from_utf8(body).unwrap();

        format!("{version} {}\n{header}\n\n{body}", self.code)
            .as_bytes()
            .to_vec()
    }
}
