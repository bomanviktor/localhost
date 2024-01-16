use crate::log;
use crate::log::LogFileType;
use crate::server::method_is_allowed;
use crate::server::path::path_exists;
use crate::server::redirections::is_redirect;
use crate::server::{Request, Route, ServerConfig, StatusCode};
use http::StatusCode as HttpStatusCode;

pub fn get_route<'a>(
    req: &'a Request<String>,
    config: &'a ServerConfig,
) -> Result<Route, (StatusCode, String)> {
    // Get the route assigned to the path
    let url_path = req.uri().path();
    let route;
    let routed_path;

    if let Some((i, path)) = path_exists(url_path.to_string(), &config.routes) {
        route = config.routes[i].clone();
        routed_path = path;
    } else {
        log!(
            LogFileType::Server,
            format!("Error: Path not found {}", url_path)
        );
        return Err((StatusCode::NOT_FOUND, "".to_string()));
    }

    // Check if it is a redirect
    if let Some(settings) = &route.settings {
        if is_redirect(url_path, &settings.http_redirections) {
            let redirect_code = settings
                .redirect_status_code
                .as_ref()
                .and_then(|code_str| code_str.parse::<u16>().ok())
                .map(|code| {
                    HttpStatusCode::from_u16(code).unwrap_or(HttpStatusCode::TEMPORARY_REDIRECT)
                })
                .unwrap_or(HttpStatusCode::TEMPORARY_REDIRECT);
            return Err((redirect_code, routed_path.to_string()));
        }
    }
    // Check if the method is allowed on route
    if !method_is_allowed(req.method(), &route) {
        log!(
            LogFileType::Server,
            format!(
                "Error: Method '{}' not allowed on path '{}'",
                req.method(),
                url_path
            )
        );

        return Err((StatusCode::METHOD_NOT_ALLOWED, "".to_string()));
    }

    Ok(route)
}
