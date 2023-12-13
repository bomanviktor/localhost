use crate::server_config::route::Route;
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

pub fn method_is_allowed(method: http::Method, route: &Route) -> bool {
    route.accepted_http_methods.contains(&method)
}

pub fn path(req: &str) -> &Path {
    let path = get_line(req, 0).split_whitespace().collect::<Vec<&str>>()[1];
    Path::new(path)
}

pub fn path_exists<'a>(
    requested_path: &'a Path,
    routes: &[Route<'a>],
) -> Option<(usize, &'a Path)> {
    for (i, route) in routes.iter().enumerate() {
        for &path in &route.paths {
            // Path is contained in the route
            if path == requested_path {
                return Some((i, path));
            }
            // Path is a part of the redirections
            if route.http_redirections.contains_key(requested_path) {
                let redirected_path = route.http_redirections.get(requested_path).unwrap();
                return path_exists(redirected_path, routes); // Recursively call itself to check for the redirected path
            }
        }
    }
    // Path does not exist in allowed paths or in redirections
    None
}

fn get_line(req: &str, line: usize) -> &str {
    req.trim_end_matches('\0')
        .split('\n')
        .collect::<Vec<&str>>()[line]
}
