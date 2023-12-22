use http::header::{COOKIE, SET_COOKIE};
use http::response::Builder;
use http::{HeaderValue, Request};

type Cookie = str;
pub fn set_cookie(resp: Builder, value: &Cookie) -> Builder {
    resp.header(SET_COOKIE, value)
}

pub fn remove_cookie(resp: Builder, value: &Cookie) -> Builder {
    let value = format!("{value}=; Expires=Thu, 01 Jan 1970 00:00:00 GMT");
    resp.header(SET_COOKIE, value)
}

/// # cookie
///
/// Adds a cookie to the response headers. Cookie is specified by `value`
pub fn cookie(resp: Builder, value: &Cookie) -> Builder {
    resp.header(COOKIE, value)
}

pub fn get_cookie<'a>(req: &'a Request<String>, value: &'a Cookie) -> Option<&'a HeaderValue> {
    req.headers()
        .get_all(COOKIE)
        .iter()
        .find(|&c| c.to_str().unwrap_or_default().contains(value))
}
