use super::*;
use std::fs;
use std::path::Path;

#[allow(dead_code)]

/// # delete_file
///
/// Delete the file specified by the `Content-Location` header.
pub fn delete_target(
    req: &Request<String>,
    conf: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    // Not authorized to delete
    if get_cookie(req, "grit-lab=cookie").is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let path = match get_target_location(req) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    // Check if target exists
    if fs::metadata(path).is_err() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Check if the target is protected
    if unsafe_delete(path) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Try to remove first the target as a file, then as a directory
    if fs::remove_file(path).is_err_and(|_| fs::remove_dir_all(path).is_err()) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    match Response::builder()
        .status(StatusCode::OK)
        .version(req.version())
        .header(HOST, conf.host)
        .body(Bytes::from("Target deleted."))
    {
        Ok(resp) => Ok(resp),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

const UNSAFE_DELETIONS: [&str; 7] = [
    "handlers",
    "default_errors",
    "cgi",
    "src",
    "server",
    "server_config",
    "localhost",
];

fn unsafe_delete(p: &Path) -> bool {
    // TODO: Make it so that you cannot delete server files
    if p.extension().is_some_and(|e| e.eq("rs")) {
        return true;
    }
    let path = p.to_str().unwrap_or_default();
    UNSAFE_DELETIONS
        .iter()
        .any(|&unsafe_path| path.contains(unsafe_path))
}
