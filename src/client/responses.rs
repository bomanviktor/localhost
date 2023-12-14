use crate::type_aliases::Bytes;
use http::StatusCode;
use std::fs;
use http::header::CONTENT_TYPE;
use crate::server_config::ServerConfig;


pub fn format(response: http::Response<()>) -> Bytes {
        let version = response.version();
        let status = response.status().as_str();
        let content_type = response.headers().get(CONTENT_TYPE).unwrap().as_bytes();

        if response.body() as Bytes.is_empty() {
            format!("{version} {status}\n{content_type}")
                .as_bytes()
                .to_vec()
        } else {
            format!("{version} {status}\n{content_type}\n\n{:?}", response.body() as Bytes)
                .as_bytes()
                .to_vec()
        }
    }

    fn content_type(path: &str) -> String {
        let file_extension = path.split('.').rev().collect::<Vec<&str>>()[0];
        // "/test.html" -> "html"

        format!(
            "Content-Type: {}",
            match file_extension {
                // Text
                "html" => "text/html",
                "css" => "text/css",
                "js" => "text/javascript",
                // Message
                "http" => "message/http",
                // Image
                "jpeg" | "jpg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",
                "bmp" => "image/bmp",
                "example" => "image/example",
                // Audio
                "aac" => "audio/aac",
                "eac3" => "audio/eac3",
                // Application
                "json" => "application/json",
                "awt" => "application/jwt",
                _ => "text/html",
            }
        )
    }


    fn check_errors(code: StatusCode, config: &ServerConfig) -> std::io::Result<Bytes> {
        let error_path =
            config
            .default_error_paths
            .get(&code)
            .expect("get wrecked");

        fs::read(format!("src/default_errors{error_path}"))
    }

pub mod redirections {
    use http::header::{HOST, LOCATION};
    use http::StatusCode;

    pub fn permanent_redirect(path: &str, host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(LOCATION, path.parse().unwrap())
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::PERMANENT_REDIRECT)
            .body(())
            .unwrap()
    }
}
pub mod client_errors {
    use http::header::HOST;
    use http::StatusCode;

    /// Returns a 400 Bad Request response.
    pub fn bad_request(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::BAD_REQUEST)
            .body(())
            .unwrap()
    }

    /// Returns a 401 Unauthorized response.
    pub fn unauthorized(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::UNAUTHORIZED)
            .body(())
            .unwrap()
    }

    /// Returns a 403 Forbidden response.
    pub fn forbidden(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::FORBIDDEN)
            .body(())
            .unwrap()
    }

    /// Returns a 404 Not Found response.
    pub fn not_found(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::NOT_FOUND)
            .body(())
            .unwrap()
    }

    /// Returns a 405 Method Not Allowed response.
    pub fn method_not_allowed(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(())
            .unwrap()
    }

    /// Returns a 409 Conflict response.
    pub fn conflict(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::CONFLICT)
            .body(())
            .unwrap()
    }

    /// Returns a 410 Gone response.
    pub fn gone(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::GONE)
            .body(())
            .unwrap()
    }

    /// Returns a 411 Length Required response.
    pub fn length_required(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::LENGTH_REQUIRED)
            .body(())
            .unwrap()
    }

    /// Returns a 412 Precondition Failed response.
    pub fn precondition_failed(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::PRECONDITION_FAILED)
            .body(())
            .unwrap()
    }

    /// Returns a 413 Payload Too Large response.
    pub fn payload_too_large(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::PAYLOAD_TOO_LARGE)
            .body(())
            .unwrap()
    }

    /// Returns a 414 URI Too Long response.
    pub fn uri_too_long(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::URI_TOO_LONG)
            .body(())
            .unwrap()
    }

    /// Returns a 415 Unsupported Media Type response.
    pub fn unsupported_media_type(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
            .body(())
            .unwrap()
    }

    /// Returns a 416 Range Not Satisfiable response.
    pub fn range_not_satisfiable(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::RANGE_NOT_SATISFIABLE)
            .body(())
            .unwrap()
    }

    /// Returns a 417 Expectation Failed response.
    pub fn expectation_failed(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::EXPECTATION_FAILED)
            .body(())
            .unwrap()
    }

    /// Returns a 418 I'm a teapot response.
    pub fn im_a_teapot(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::IM_A_TEAPOT)
            .body(())
            .unwrap()
    }

    /// Returns a 421 Misdirected Request response.
    pub fn misdirected_request(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::MISDIRECTED_REQUEST)
            .body(())
            .unwrap()
    }

    /// Returns a 422 Unprocessable Entity response.
    pub fn unprocessable_entity(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(())
            .unwrap()
    }

    /// Returns a 423 Locked response.
    pub fn locked(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::LOCKED)
            .body(())
            .unwrap()
    }

    /// Returns a 424 Failed Dependency response.
    pub fn failed_dependency(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::FAILED_DEPENDENCY)
            .body(())
            .unwrap()
    }

    /// Returns a 425 Too Early response.
    pub fn too_early(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::TOO_EARLY)
            .body(())
            .unwrap()
    }

    /// Returns a 426 Upgrade Required response.
    pub fn upgrade_required(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::UPGRADE_REQUIRED)
            .body(())
            .unwrap()
    }

    /// Returns a 428 Precondition Required response.
    pub fn precondition_required(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::PRECONDITION_REQUIRED)
            .body(())
            .unwrap()
    }

    /// Returns a 429 Too Many Requests response.
    pub fn too_many_requests(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(())
            .unwrap()
    }

    /// Returns a 431 Request Header Fields Too Large response.
    pub fn request_header_fields_too_large(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE)
            .body(())
            .unwrap()
    }

    /// Returns a 451 Unavailable For Legal Reasons response.
    pub fn unavailable_for_legal_reasons(host: &str) -> http::Response<()> {
        http::Response::builder()
            .header(HOST, host.parse().unwrap())
            .status(StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS)
            .body(())
            .unwrap()
    }


}

pub mod server_errors {

}