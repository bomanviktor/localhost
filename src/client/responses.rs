use crate::client::utils::to_bytes;
use crate::server_config::ServerConfig;
use crate::type_aliases::Bytes;
use http::{Response, StatusCode, Version};
use std::fmt::Display;
use std::fs;

pub fn format<T: ToString + Display>(response: Response<T>) -> Bytes {
    let version = response.version();
    let binding = response.status();
    let status = binding.as_str();
    let mut resp = format!("{version:?} {status}\n");

    // Get all headers into the response
    for (name, value) in response.headers() {
        let name = name.to_string();
        let value = value.to_str().unwrap();
        let header = format!("{name}: {value}\n");
        resp.push_str(&header);
    }

    let body = response.body().to_string();
    if !body.is_empty() {
        resp.push_str(&format!("\n{body}"));
    }

    to_bytes(&resp)
}

pub fn content_type(path: &str) -> String {
    let file_extension = path.split('.').rev().collect::<Vec<&str>>()[0];
    // "/test.html" -> "html"

    format!(
        "Content-Type: {}",
        match file_extension {
            // Text
            "html" => "text/html",
            "css" => "text/css",
            "js" => "text/javascript",
            "txt" => "text/plain",
            "xml" => "text/xml",
            // Message
            "http" => "message/http",
            // Image
            "jpeg" | "jpg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            "svg" => "image/svg+xml",
            // Audio
            "aac" => "audio/aac",
            "eac3" => "audio/eac3",
            "mp3" => "audio/mpeg",
            "ogg" => "audio/ogg",
            // Video
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "ogv" => "video/ogg",
            // Application
            "json" => "application/json",
            "pdf" => "application/pdf",
            "zip" => "application/zip",
            "tar" => "application/x-tar",
            "gz" => "application/gzip",
            "exe" => "application/octet-stream",
            "msi" => "application/octet-stream",
            "woff" => "application/font-woff",
            "woff2" => "application/font-woff2",
            "ttf" => "application/font-sfnt",
            "otf" => "application/font-sfnt",
            // Default to HTML for unknown types
            _ => "text/html",
        }
    )
}
fn check_errors(code: StatusCode, config: &ServerConfig) -> std::io::Result<Bytes> {
    let error_path = config
        .default_error_paths
        .get(&code)
        .unwrap_or(&"/400.html");
    fs::read(format!("src/default_errors{error_path}"))
}

pub mod informational {
    use super::*;
    use http::header::{HOST, SERVER};

    fn base_response(host: &str, version: Version, status: StatusCode) -> Response<String> {
        http::Response::builder()
            .version(version)
            .header(HOST, host) // Replace with your actual header
            .header(SERVER, "grit:lab-localhost/1.0") // Replace with your actual server name and version
            .status(status)
            .body("".to_string())
            .unwrap()
    }

    /// Returns a 100 Continue response.
    pub fn continue_response(host: &str, version: Version) -> Response<String> {
        base_response(host, version, StatusCode::CONTINUE)
    }

    /// Returns a 101 Switching Protocols response.
    pub fn switching_protocols(host: &str, version: Version) -> Response<String> {
        base_response(host, version, StatusCode::SWITCHING_PROTOCOLS)
    }

    /// Returns a 102 Processing response.
    pub fn processing(host: &str, version: Version) -> Response<String> {
        base_response(host, version, StatusCode::PROCESSING)
    }

    /// Returns a 103 Early Hints response.
    pub fn early_hints(host: &str, version: Version) -> Response<String> {
        base_response(host, version, StatusCode::from_u16(103).unwrap())
    }

    /// Returns a 104 response (Unofficial - Used by a web accelerator).
    pub fn unofficial_web_accelerator(host: &str, version: Version) -> Response<String> {
        base_response(host, version, StatusCode::from_u16(104).unwrap())
    }
}

pub mod redirections {
    use super::*;
    use http::header::{HOST, LOCATION, SERVER};

    fn base_response(
        host: &str,
        version: Version,
        status: StatusCode,
        path: &str,
    ) -> Response<String> {
        http::Response::builder()
            .version(version)
            .header(HOST, host) // Replace with your actual header
            .header(SERVER, "grit:lab-localhost/1.0")
            .header(LOCATION, path)
            .status(status)
            .body("".to_string())
            .unwrap()
    }

    /// Returns a 301 Moved Permanently response with the specified location.
    pub fn moved_permanently(host: &str, path: &str, version: Version) -> Response<String> {
        base_response(host, version, StatusCode::MOVED_PERMANENTLY, path)
    }

    /// Returns a 302 Found response with the specified location.
    pub fn found(host: &str, path: &str, version: Version) -> Response<String> {
        base_response(host, version, StatusCode::FOUND, path)
    }

    /// Returns a 303 See Other response with the specified location.
    pub fn see_other(host: &str, path: &str, version: Version) -> Response<String> {
        base_response(host, version, StatusCode::SEE_OTHER, path)
    }

    /// Returns a 307 Temporary Redirect response with the specified location.
    pub fn temporary_redirect(host: &str, path: &str, version: Version) -> Response<String> {
        base_response(host, version, StatusCode::TEMPORARY_REDIRECT, path)
    }

    /// Returns a 308 Permanent Redirect response with the specified location.
    pub fn permanent_redirect(host: &str, path: &str, version: Version) -> Response<String> {
        base_response(host, version, StatusCode::PERMANENT_REDIRECT, path)
    }
}

pub mod errors {
    use super::*;
    use crate::client::responses::check_errors;
    use crate::client::utils::to_bytes;
    use crate::server_config::ServerConfig;
    use http::header::{HOST, SERVER};

    fn base_response(
        config: &ServerConfig,
        version: Version,
        code: StatusCode,
    ) -> Response<String> {
        let error_body = match check_errors(code, config) {
            Ok(body) => body,
            Err(_) => to_bytes("400"),
        };

        Response::builder()
            .version(version)
            .header(HOST, config.host)
            .header(SERVER, "grit:lab-localhost/1.0")
            .status(code)
            .body(String::from_utf8(error_body).unwrap())
            .unwrap()
    }
    pub mod client_errors {
        use super::*;
        use crate::server_config::ServerConfig;
        /// Returns a 400 Bad Request response.
        /// Returns a 400 Bad Request response.
        pub fn bad_request(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::BAD_REQUEST)
        }

        /// Returns a 401 Unauthorized response.
        pub fn unauthorized(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::UNAUTHORIZED)
        }

        /// Returns a 403 Forbidden response.
        pub fn forbidden(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::FORBIDDEN)
        }

        /// Returns a 404 Not Found response.
        pub fn not_found(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::NOT_FOUND)
        }

        /// Returns a 405 Method Not Allowed response.
        pub fn method_not_allowed(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::METHOD_NOT_ALLOWED)
        }

        /// Returns a 409 Conflict response.
        pub fn conflict(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::CONFLICT)
        }

        /// Returns a 410 Gone response.
        pub fn gone(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::GONE)
        }

        /// Returns a 411 Length Required response.
        pub fn length_required(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::LENGTH_REQUIRED)
        }

        /// Returns a 412 Precondition Failed response.
        pub fn precondition_failed(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::PRECONDITION_FAILED)
        }

        /// Returns a 413 Payload Too Large response.
        pub fn payload_too_large(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::PAYLOAD_TOO_LARGE)
        }

        /// Returns a 414 URI Too Long response.
        pub fn uri_too_long(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::URI_TOO_LONG)
        }

        /// Returns a 415 Unsupported Media Type response.
        pub fn unsupported_media_type(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::UNSUPPORTED_MEDIA_TYPE)
        }

        /// Returns a 416 Range Not Satisfiable response.
        pub fn range_not_satisfiable(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::RANGE_NOT_SATISFIABLE)
        }

        /// Returns a 417 Expectation Failed response.
        pub fn expectation_failed(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::EXPECTATION_FAILED)
        }

        /// Returns a 418 I'm a teapot response.
        pub fn im_a_teapot(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::IM_A_TEAPOT)
        }

        /// Returns a 421 Misdirected Request response.
        pub fn misdirected_request(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::MISDIRECTED_REQUEST)
        }

        /// Returns a 422 Unprocessable Entity response.
        pub fn unprocessable_entity(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::UNPROCESSABLE_ENTITY)
        }

        /// Returns a 423 Locked response.
        pub fn locked(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::LOCKED)
        }

        /// Returns a 424 Failed Dependency response.
        pub fn failed_dependency(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::FAILED_DEPENDENCY)
        }

        /// Returns a 425 Too Early response.
        pub fn too_early(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::from_u16(425).unwrap())
        }

        /// Returns a 426 Upgrade Required response.
        pub fn upgrade_required(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::UPGRADE_REQUIRED)
        }

        /// Returns a 428 Precondition Required response.
        pub fn precondition_required(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::PRECONDITION_REQUIRED)
        }

        /// Returns a 429 Too Many Requests response.
        pub fn too_many_requests(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::TOO_MANY_REQUESTS)
        }

        /// Returns a 431 Request Header Fields Too Large response.
        pub fn request_header_fields_too_large(
            config: &ServerConfig,
            version: Version,
        ) -> Response<String> {
            base_response(config, version, StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE)
        }

        /// Returns a 451 Unavailable For Legal Reasons response.
        pub fn unavailable_for_legal_reasons(
            config: &ServerConfig,
            version: Version,
        ) -> Response<String> {
            base_response(config, version, StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS)
        }
    }

    pub mod server_errors {
        use super::*;

        /// Returns a 500 Internal Server Error response.
        pub fn internal_server_error(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::INTERNAL_SERVER_ERROR)
        }

        /// Returns a 501 Not Implemented response.
        pub fn not_implemented(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::NOT_IMPLEMENTED)
        }

        /// Returns a 502 Bad Gateway response.
        pub fn bad_gateway(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::BAD_GATEWAY)
        }

        /// Returns a 503 Service Unavailable response.
        pub fn service_unavailable(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::SERVICE_UNAVAILABLE)
        }

        /// Returns a 504 Gateway Timeout response.
        pub fn gateway_timeout(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::GATEWAY_TIMEOUT)
        }

        /// Returns a 505 HTTP Version Not Supported response.
        pub fn http_version_not_supported(
            config: &ServerConfig,
            version: Version,
        ) -> Response<String> {
            base_response(config, version, StatusCode::HTTP_VERSION_NOT_SUPPORTED)
        }

        /// Returns a 506 Variant Also Negotiates response.
        pub fn variant_also_negotiates(
            config: &ServerConfig,
            version: Version,
        ) -> Response<String> {
            base_response(config, version, StatusCode::VARIANT_ALSO_NEGOTIATES)
        }

        /// Returns a 507 Insufficient Storage response.
        pub fn insufficient_storage(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::INSUFFICIENT_STORAGE)
        }

        /// Returns a 508 Loop Detected response.
        pub fn loop_detected(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::LOOP_DETECTED)
        }

        /// Returns a 510 Not Extended response.
        pub fn not_extended(config: &ServerConfig, version: Version) -> Response<String> {
            base_response(config, version, StatusCode::NOT_EXTENDED)
        }

        /// Returns a 511 Network Authentication Required response.
        pub fn network_authentication_required(
            config: &ServerConfig,
            version: Version,
        ) -> Response<String> {
            base_response(config, version, StatusCode::NETWORK_AUTHENTICATION_REQUIRED)
        }
    }
}
