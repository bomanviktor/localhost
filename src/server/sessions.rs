use crate::server::content_type;
use crate::server_config::ServerConfig;
use crate::type_aliases::Bytes;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE, COOKIE, HOST, SET_COOKIE};
use http::response::Builder;
use http::{HeaderValue, Request, Response, StatusCode};
use std::fs;

type Cookie = str;

pub fn update_cookie(
    req: &Request<Bytes>,
    conf: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    if req.headers().iter().any(|(_, v)| {
        v.to_str()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .eq("grit:lab=cookie")
    }) {
        return remove_cookie(
            Response::builder()
                .status(StatusCode::OK)
                .version(req.version()),
            "grit:lab=cookie",
        ) // Replace this with a database value.
        .header(HOST, conf.host)
        .body(vec![])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    }

    set_cookie(
        Response::builder()
            .status(StatusCode::OK)
            .version(req.version()),
        "grit:lab=cookie",
    ) // Replace this with a database value.
    .header(HOST, conf.host)
    .body(vec![])
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn validate_cookie(
    req: &Request<Bytes>,
    conf: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    let value = get_cookie(req, "grit:lab=cookie")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .unwrap_or_default();

    cookie(
        Response::builder()
            .status(StatusCode::OK)
            .version(req.version()),
        value,
    )
    .header(HOST, conf.host)
    .body(vec![])
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn cookie_demo(
    req: &Request<Bytes>,
    config: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    let body = fs::read("./files/cookie-demo.html").map_err(|_| StatusCode::NOT_FOUND)?;
    let mut resp = Response::builder()
        .version(req.version())
        .header(HOST, config.host)
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type("./files/cookie_demo.html"))
        .header(CONTENT_LENGTH, body.len());

    for (key, value) in req.headers() {
        if crate::server::methods::safe::STANDARD_HEADERS.contains(key) {
            resp = resp.header(key, value);
        }
    }

    resp.body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn set_cookie(resp: Builder, value: &Cookie) -> Builder {
    let value = format!("{value}; path=/; Max-Age=3600"); // Expires in 1 hour
    resp.header(SET_COOKIE, value)
}

pub fn remove_cookie(resp: Builder, value: &Cookie) -> Builder {
    let value = format!("{value}; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT");
    resp.header(SET_COOKIE, value)
}

/// # cookie
///
/// Adds a cookie to the response headers. Cookie is specified by `value`
pub fn cookie(resp: Builder, value: &Cookie) -> Builder {
    resp.header(COOKIE, value)
}

pub fn get_cookie<'a>(req: &'a Request<Bytes>, value: &'a Cookie) -> Option<&'a HeaderValue> {
    req.headers()
        .get_all(COOKIE)
        .iter()
        .find(|&c| c.to_str().unwrap_or_default().eq(value))
}
