use super::*;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use std::fs;

/// # upload_content
///
/// Uploads content in request body to server.
#[allow(dead_code)]
pub fn upload_content(
    req: &Request<String>,
    conf: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    if get_cookie(req, "grit-lab=cookie").is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if req.body().len() > conf.body_size_limit {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    let content_type = match req.headers().get(CONTENT_TYPE) {
        Some(ct) => ct,
        None => return Err(StatusCode::BAD_REQUEST),
    };

    if req.headers().get(CONTENT_LENGTH).is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let path = match get_target_location(req) {
        Ok(p) => p,
        Err(code) => return Err(code),
    };

    if fs::write(path, req.body()).is_err() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let body = format!("{:?} was successfully uploaded", path.file_name().unwrap());
    match Response::builder()
        .version(req.version())
        .status(StatusCode::CREATED)
        .header(CONTENT_TYPE, content_type)
        .body(Bytes::from(body))
    {
        Ok(resp) => Ok(resp),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
