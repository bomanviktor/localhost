use super::*;
use crate::server::{cookie, get_cookie, remove_cookie, set_cookie};

pub fn add_cookie(
    req: &Request<String>,
    conf: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    let resp = Response::builder()
        .status(StatusCode::OK)
        .version(req.version());

    match set_cookie(resp, "grit-lab=cookie") // Replace this with a database value.
        .header(HOST, conf.host)
        .body(vec![])
    {
        Ok(resp) => Ok(resp),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub fn delete_cookie(
    req: &Request<String>,
    conf: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    match remove_cookie(
        Response::builder()
            .status(StatusCode::OK)
            .version(req.version()),
        "grit-lab",
    ) // Replace this with a database value.
    .header(HOST, conf.host)
    .body(vec![])
    {
        Ok(resp) => Ok(resp),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub fn validate_cookie(
    req: &Request<String>,
    conf: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    let value = match get_cookie(req, "grit-lab=cookie") {
        // Replace this with a database value.
        Some(c) => c.to_str().unwrap_or_default(),
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    if value.is_empty() {
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
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
