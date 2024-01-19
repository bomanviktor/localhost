use crate::log;
use crate::log::*;
use crate::server_config::ServerConfig;
use crate::type_aliases::Bytes;
use http::header::{COOKIE, HOST, SET_COOKIE};
use http::response::Builder;
use http::{HeaderValue, Request, Response, StatusCode};

type Cookie = str;

pub fn update_cookie(
    req: &Request<Bytes>,
    conf: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    // Replace with
    if req
        .headers()
        .iter()
        .any(|(_, v)| v.to_str().unwrap().contains("cookie"))
    {
        return match remove_cookie(
            Response::builder()
                .status(StatusCode::OK)
                .version(req.version()),
            "grit:lab-cookie",
        ) // Replace this with a database value.
        .header(HOST, conf.host)
        .body(vec![])
        {
            Ok(resp) => Ok(resp),
            Err(_) => {
                log!(
                    LogFileType::Server,
                    "Error: Failed to remove cookie".to_string()
                ); //ToDo:format with cookie value
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        };
    }

    match set_cookie(
        Response::builder()
            .status(StatusCode::OK)
            .version(req.version()),
        "grit:lab-cookie",
    ) // Replace this with a database value.
    .header(HOST, conf.host)
    .body(vec![])
    {
        Ok(resp) => Ok(resp),
        Err(_) => {
            log!(
                LogFileType::Server,
                "Error: Failed to set cookie".to_string()
            ); //ToDo:format with cookie value
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub fn validate_cookie(
    req: &Request<Bytes>,
    conf: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    let value = match get_cookie(req, "grit:lab-cookie") {
        // Replace this with a database value.
        Some(c) => c.to_str().unwrap_or_default(),
        None => {
            log!(
                LogFileType::Server,
                "Error: Failed to get cookie".to_string()
            ); //ToDo:format with cookie value
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    if value.is_empty() {
        log!(LogFileType::Server, "Error: cookie value empty".to_string()); //ToDo:format with cookie value
        return Err(StatusCode::UNAUTHORIZED);
    }

    match cookie(
        Response::builder()
            .status(StatusCode::OK)
            .version(req.version()),
        value,
    )
    .header(HOST, conf.host)
    .body(vec![])
    {
        Ok(resp) => Ok(resp),
        Err(_) => {
            log!(
                LogFileType::Server,
                "Error: Failed to validate cookie".to_string()
            ); //ToDo:format with cookie value
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub fn set_cookie(resp: Builder, value: &Cookie) -> Builder {
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
        .find(|&c| c.to_str().unwrap_or_default().contains(value))
}
