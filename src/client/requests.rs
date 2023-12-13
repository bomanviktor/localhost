use std::path::Path;

pub fn method(req: &str) -> http::Method {
    let method = get_line(req, 0).split_whitespace().collect::<Vec<&str>>()[0];
    match method {
        "GET" => http::Method::GET,
        "HEAD" => http::Method::HEAD,
        "POST" => http::Method::POST,
        "PUT" => http::Method::PUT,
        "DELETE" => http::Method::DELETE,
        "CONNECT" => http::Method::CONNECT,
        "TRACE" => http::Method::TRACE,
        "OPTIONS" => http::Method::OPTIONS,
        "PATCH" => http::Method::PATCH,
        _ => http::Method::GET, // PERHAPS RETURN ERROR HERE.
    }
}

pub fn method_is_allowed(method: http::Method, allowed_methods: &[http::Method]) -> bool {
    allowed_methods.contains(&method)
}

pub fn path(req: &str) -> &Path {
    let path = get_line(req, 0).split_whitespace().collect::<Vec<&str>>()[1];
    Path::new(path)
}

fn get_line(req: &str, line: usize) -> &str {
    req.trim_end_matches('\0')
        .split('\n')
        .collect::<Vec<&str>>()[line]
}
