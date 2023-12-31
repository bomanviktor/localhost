use crate::log;
use crate::log::LogFileType;
use crate::server::method_is_allowed;
use crate::server::path::path_exists;
use crate::server::redirections::is_redirect;
use crate::server::{Request, Route, ServerConfig, StatusCode};

pub fn get_route<'a>(
    req: &'a Request<String>,
    config: &'a ServerConfig,
) -> Result<Route<'a>, (StatusCode, String)> {
    // Get the route assigned to the path
    let url_path = req.uri().path();
    let route;
    let routed_path;

    if let Some((i, path)) = path_exists(url_path, &config.routes) {
        route = config.routes[i].clone();
        routed_path = path;
    } else {
        log!(
            LogFileType::Server,
            format!("Error: Path not found {}", url_path)
        );
        return Err((StatusCode::NOT_FOUND, "".to_string()));
    }

    if is_redirect(url_path, routed_path) {
        log!(
            LogFileType::Server,
            format!("Error: Path not redirect: {} != {}", url_path, routed_path)
        );
        return Err((
            route.settings.unwrap().redirect_status_code,
            routed_path.to_string(),
        ));
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
