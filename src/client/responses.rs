use crate::server_config::ServerConfig;
use crate::type_aliases::Bytes;
use http::{Method, StatusCode};
use std::fs;

pub struct Response<'a> {
    code: StatusCode,
    method: Method,
    path: &'a str,
    config: &'a ServerConfig<'a>,
    bytes: Option<Bytes>,
}

impl<'a> Response<'a> {
    pub fn new(
        code: StatusCode,
        method: &Method,
        path: &'a str,
        config: &'a ServerConfig<'a>,
        bytes: Option<Bytes>,
    ) -> Self {
        Self {
            code,
            method: method.clone(),
            path,
            config,
            bytes,
        }
    }
    pub fn format(&self) -> Bytes {
        let header = "Content-Type: text/html";
        let version = "HTTP/1.1"; // Change this to get the version from the request

        let body = match self.code {
            StatusCode::OK | StatusCode::PERMANENT_REDIRECT => self.check_method(),
            _ => Some(self.check_errors().unwrap()),
        };

        if let Some(body) = body {
            let body = String::from_utf8(body).unwrap();
            format!("{version} {}\n{header}\n\n{body}", self.code)
                .as_bytes()
                .to_vec()
        } else {
            format!("{version} {}\n{header}", self.code)
                .as_bytes()
                .to_vec()
        }
    }

    fn check_method(&self) -> Option<Bytes> {
        match self.method {
            Method::GET => Some(self.get().unwrap_or_default()),
            Method::HEAD => None,
            Method::POST => {
                self.post().unwrap_or_default();
                None
            }
            Method::PUT => {
                self.put().unwrap_or_default();
                None
            }
            Method::PATCH => {
                self.patch().unwrap_or_default();
                None
            }
            Method::DELETE => {
                self.delete().unwrap_or_default();
                None
            }
            _ => Some(self.get().unwrap()),
        }
    }

    fn check_errors(&self) -> std::io::Result<Bytes> {
        let error_path = self
            .config
            .default_error_paths
            .get(&self.code)
            .expect("get wrecked");

        fs::read(format!("src/default_errors{error_path}"))
    }
    fn get(&self) -> std::io::Result<Bytes> {
        fs::read(self.path)
    }

    fn post(&self) -> std::io::Result<()> {
        // Resource does not exist, so create it.
        if fs::read(self.path).is_err() {
            return fs::write(self.path, self.bytes.clone().unwrap());
        }

        let mut path = String::from(self.path); // Clone the original path to avoid modifying it directly.
        let end = path.rfind('.').unwrap_or(path.len());
        let mut i = 1;

        // If the file already exists, modify the path.
        // /foo.txt -> /foo(1).txt
        while fs::metadata(&path).is_ok() {
            path.truncate(end);
            path.push_str(&format!("({})", i));
            path.push_str(&self.path[end..]);
            i += 1;
        }

        fs::write(&path, self.bytes.clone().unwrap())
    }

    fn put(&self) -> std::io::Result<()> {
        fs::write(self.path, self.bytes.clone().unwrap())
    }

    fn patch(&self) -> std::io::Result<()> {
        fs::write(self.path, self.bytes.clone().unwrap())
    }

    fn delete(&self) -> std::io::Result<()> {
        match fs::remove_file(self.path) {
            Ok(_) => Ok(()),
            Err(_) => fs::remove_file(self.path),
        }
    }
}
