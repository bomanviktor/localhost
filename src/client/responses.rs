use crate::server_config::ServerConfig;
use crate::type_aliases::Bytes;
use http::{HeaderMap, Method, StatusCode};
use std::fs;

pub struct Response<'a> {
    header: HeaderMap,
    code: StatusCode,
    method: Method,
    path: &'a str,
    config: &'a ServerConfig<'a>,
    bytes: Option<Bytes>,
}

impl<'a> Response<'a> {
    pub fn new(
        header: HeaderMap,
        code: StatusCode,
        method: &Method,
        path: &'a str,
        config: &'a ServerConfig<'a>,
        bytes: Option<Bytes>,
    ) -> Self {
        Self {
            header,
            code,
            method: method.clone(),
            path,
            config,
            bytes,
        }
    }
    pub fn format(&self) -> Bytes {
        println!("{:?}", self.header);
        let content_type = self.content_type();
        let version = "HTTP/1.1"; // Change this to get the version from the request
        let code = self.code;

        let body = match self.code {
            StatusCode::OK | StatusCode::PERMANENT_REDIRECT => self.check_method(),
            _ => Some(self.check_errors().unwrap()),
        };

        if let Some(body) = body {
            let body = String::from_utf8(body).unwrap();
            format!("{version} {code}\n{content_type}\n\n{body}")
                .as_bytes()
                .to_vec()
        } else {
            format!("{version} {code}\n{content_type}")
                .as_bytes()
                .to_vec()
        }
    }

    fn content_type(&self) -> String {
        let file_extension = self.path.split('.').rev().collect::<Vec<&str>>()[0];
        // "/test.html" -> "html"

        format!(
            "Content-Type: {}",
            match file_extension {
                // Text
                "html" => "text/html",
                "css" => "text/css",
                "js" => "text/javascript",
                // Message
                "http" => "message/http",
                // Image
                "jpeg" | "jpg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",
                "bmp" => "image/bmp",
                "example" => "image/example",
                // Audio
                "aac" => "audio/aac",
                "eac3" => "audio/eac3",
                // Application
                "json" => "application/json",
                "awt" => "application/jwt",
                _ => "text/html",
            }
        )
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
}

mod request_method {
    use super::*;
    impl<'a> Response<'a> {
        pub fn get(&self) -> std::io::Result<Bytes> {
            fs::read(self.path)
        }

        pub fn post(&self) -> std::io::Result<()> {
            // Resource does not exist, so create it.
            if fs::metadata(self.path).is_err() {
                return fs::write(self.path, self.bytes.clone().unwrap());
            }

            let mut path = String::from(self.path); // Turn the path into String
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

        pub fn put(&self) -> std::io::Result<()> {
            fs::write(self.path, self.bytes.clone().unwrap())
        }

        pub fn patch(&self) -> std::io::Result<()> {
            match fs::metadata(self.path) {
                Ok(_) => fs::write(self.path, self.bytes.clone().unwrap()),
                Err(e) => Err(e),
            }
        }

        pub fn delete(&self) -> std::io::Result<()> {
            match fs::remove_file(self.path) {
                Ok(_) => Ok(()),
                Err(_) => fs::remove_file(self.path),
            }
        }
    }
}
