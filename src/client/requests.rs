use crate::server_config::ServerConfig;

pub fn method(req: &str, _config: &ServerConfig) -> http::Method {
    let formatted_req: &str = req
        .trim_end_matches('\0')
        .split('\n')
        .collect::<Vec<&str>>()[0];
    let method = formatted_req.split(' ').collect::<Vec<&str>>()[0];

    match method {
        "GET" => http::Method::GET,
        "POST" => http::Method::POST,
        "DELETE" => http::Method::DELETE,
        _ => http::Method::OPTIONS,
    }
}
