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

pub mod informational {
    use http::header::{HOST, SERVER};
    use http::response::Builder;
    use http::{Response, StatusCode, Version};

    fn base_response(
        host: &str,
        version: Version,
        status: StatusCode,
    ) -> Builder {
        http::Response::builder()
            .version(version)
            .header(HOST, host.parse().unwrap())
            .header(SERVER, "YourServerName/1.0") // Replace with your actual server name and version
            .status(status)
    }

    /// Returns a 100 Continue response.
    pub fn continue_response(host: &str, version: Version) -> Response<()> {
        base_response(host, version, StatusCode::CONTINUE)
            .body(())
            .unwrap()
    }

    /// Returns a 101 Switching Protocols response.
    pub fn switching_protocols(host: &str, version: Version) -> Response<()> {
        base_response(host, version, StatusCode::SWITCHING_PROTOCOLS)
            .body(())
            .unwrap()
    }

    /// Returns a 102 Processing response.
    pub fn processing(host: &str, version: Version) -> Response<()> {
        base_response(host, version, StatusCode::PROCESSING)
            .body(())
            .unwrap()
    }

    /// Returns a 103 Early Hints response.
    pub fn early_hints(host: &str, version: Version) -> Response<()> {
        base_response(host, version, StatusCode::EARLY_HINTS)
            .body(())
            .unwrap()
    }

    /// Returns a 104 response (Unofficial - Used by a web accelerator).
    pub fn unofficial_web_accelerator(host: &str, version: Version) -> Response<()> {
        base_response(host, version, StatusCode::from_u16(104).unwrap())
            .body(())
            .unwrap()
    }
}



pub mod redirections {
    use http::header::{HOST, LOCATION, SERVER};
    use http::{Response, StatusCode, Version};
    use http::response::Builder;

    fn base_response(
        host: &str,
        version: Version,
        status: StatusCode,
    ) -> Builder {
        http::Response::builder()
            .version(version)
            .header(HOST, host.parse().unwrap())
            .header(SERVER, "grit:lab-localhost/1.0") // Replace with your actual server name and version
            .status(status)
    }

    /// Returns a 301 Moved Permanently response with the specified location.
    pub fn moved_permanently(host: &str, path: &str, version: Version) -> Response<()> {
        base_response(host, version, StatusCode::MOVED_PERMANENTLY)
            .header(LOCATION, path.parse().unwrap())
            .body(())
            .unwrap()
    }

    /// Returns a 302 Found response with the specified location.
    pub fn found(host: &str, path: &str, version: Version) -> Response<()> {
        base_response(host, version, StatusCode::FOUND)
            .header(LOCATION, path.parse().unwrap())
            .body(())
            .unwrap()
    }

    /// Returns a 303 See Other response with the specified location.
    pub fn see_other(host: &str, path: &str, version: Version) -> Response<()> {
        base_response(host, version, StatusCode::SEE_OTHER)
            .header(LOCATION, path.parse().unwrap())
            .body(())
            .unwrap()
    }

    /// Returns a 307 Temporary Redirect response with the specified location.
    pub fn temporary_redirect(host: &str, path: &str, version: Version) -> Response<()> {
        base_response(host, version, StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, path.parse().unwrap())
            .body(())
            .unwrap()
    }

    /// Returns a 308 Permanent Redirect response with the specified location.
    pub fn permanent_redirect(host: &str, path: &str, version: Version) -> Response<()> {
        base_response(host, version, StatusCode::PERMANENT_REDIRECT)
            .header(LOCATION, path.parse().unwrap())
            .body(())
            .unwrap()
    }
}

pub mod errors {
    use http::header::{HOST, SERVER};
    use http::{Response, StatusCode, Version};
    use http::response::Builder;

    fn base_response(host: &str, version: Version, status: StatusCode) -> Builder {
        Response::builder()
            .version(version)
            .header(HOST, host.parse().unwrap())
            .header(SERVER, "grit:lab-localhost/1.0") // Replace with your actual server name and version
            .status(status)
    }
    pub mod client_errors {
        use super::*;
        /// Returns a 400 Bad Request response.
        pub fn bad_request(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::BAD_REQUEST).body(()).unwrap()
        }

        /// Returns a 401 Unauthorized response.
        pub fn unauthorized(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::UNAUTHORIZED).body(()).unwrap()
        }

        /// Returns a 403 Forbidden response.
        pub fn forbidden(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::FORBIDDEN).body(()).unwrap()
        }

        /// Returns a 404 Not Found response.
        pub fn not_found(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::NOT_FOUND).body(()).unwrap()
        }

        /// Returns a 405 Method Not Allowed response.
        pub fn method_not_allowed(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::METHOD_NOT_ALLOWED).body(()).unwrap()
        }

        /// Returns a 409 Conflict response.
        pub fn conflict(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::CONFLICT).body(()).unwrap()
        }

        /// Returns a 410 Gone response.
        pub fn gone(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::GONE).body(()).unwrap()
        }

        /// Returns a 411 Length Required response.
        pub fn length_required(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::LENGTH_REQUIRED).body(()).unwrap()
        }

        /// Returns a 412 Precondition Failed response.
        pub fn precondition_failed(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::PRECONDITION_FAILED).body(()).unwrap()
        }

        /// Returns a 413 Payload Too Large response.
        pub fn payload_too_large(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::PAYLOAD_TOO_LARGE).body(()).unwrap()
        }

        /// Returns a 414 URI Too Long response.
        pub fn uri_too_long(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::URI_TOO_LONG).body(()).unwrap()
        }

        /// Returns a 415 Unsupported Media Type response.
        pub fn unsupported_media_type(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::UNSUPPORTED_MEDIA_TYPE).body(()).unwrap()
        }

        /// Returns a 416 Range Not Satisfiable response.
        pub fn range_not_satisfiable(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::RANGE_NOT_SATISFIABLE).body(()).unwrap()
        }

        /// Returns a 417 Expectation Failed response.
        pub fn expectation_failed(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::EXPECTATION_FAILED).body(()).unwrap()
        }

        /// Returns a 418 I'm a teapot response.
        pub fn im_a_teapot(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::IM_A_TEAPOT).body(()).unwrap()
        }

        /// Returns a 421 Misdirected Request response.
        pub fn misdirected_request(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::MISDIRECTED_REQUEST).body(()).unwrap()
        }

        /// Returns a 422 Unprocessable Entity response.
        pub fn unprocessable_entity(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::UNPROCESSABLE_ENTITY).body(()).unwrap()
        }

        /// Returns a 423 Locked response.
        pub fn locked(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::LOCKED).body(()).unwrap()
        }

        /// Returns a 424 Failed Dependency response.
        pub fn failed_dependency(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::FAILED_DEPENDENCY).body(()).unwrap()
        }

        /// Returns a 425 Too Early response.
        pub fn too_early(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::TOO_EARLY).body(()).unwrap()
        }

        /// Returns a 426 Upgrade Required response.
        pub fn upgrade_required(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::UPGRADE_REQUIRED).body(()).unwrap()
        }

        /// Returns a 428 Precondition Required response.
        pub fn precondition_required(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::PRECONDITION_REQUIRED).body(()).unwrap()
        }

        /// Returns a 429 Too Many Requests response.
        pub fn too_many_requests(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::TOO_MANY_REQUESTS).body(()).unwrap()
        }

        /// Returns a 431 Request Header Fields Too Large response.
        pub fn request_header_fields_too_large(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE)
                .body(())
                .unwrap()
        }

        /// Returns a 451 Unavailable For Legal Reasons response.
        pub fn unavailable_for_legal_reasons(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS)
                .body(())
                .unwrap()
        }
    }



    pub mod server_errors {
        use super::*;


        /// Returns a 500 Internal Server Error response.
        pub fn internal_server_error(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::INTERNAL_SERVER_ERROR)
                .body(())
                .unwrap()
        }

        /// Returns a 501 Not Implemented response.
        pub fn not_implemented(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::NOT_IMPLEMENTED)
                .body(())
                .unwrap()
        }

        /// Returns a 502 Bad Gateway response.
        pub fn bad_gateway(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::BAD_GATEWAY)
                .body(())
                .unwrap()
        }

        /// Returns a 503 Service Unavailable response.
        pub fn service_unavailable(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::SERVICE_UNAVAILABLE)
                .body(())
                .unwrap()
        }

        /// Returns a 504 Gateway Timeout response.
        pub fn gateway_timeout(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::GATEWAY_TIMEOUT)
                .body(())
                .unwrap()
        }

        /// Returns a 505 HTTP Version Not Supported response.
        pub fn http_version_not_supported(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::HTTP_VERSION_NOT_SUPPORTED)
                .body(())
                .unwrap()
        }

        /// Returns a 506 Variant Also Negotiates response.
        pub fn variant_also_negotiates(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::VARIANT_ALSO_NEGOTIATES)
                .body(())
                .unwrap()
        }

        /// Returns a 507 Insufficient Storage response.
        pub fn insufficient_storage(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::INSUFFICIENT_STORAGE)
                .body(())
                .unwrap()
        }

        /// Returns a 508 Loop Detected response.
        pub fn loop_detected(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::LOOP_DETECTED)
                .body(())
                .unwrap()
        }

        /// Returns a 510 Not Extended response.
        pub fn not_extended(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::NOT_EXTENDED)
                .body(())
                .unwrap()
        }

        /// Returns a 511 Network Authentication Required response.
        pub fn network_authentication_required(host: &str, version: Version) -> Response<()> {
            base_response(host, version, StatusCode::NETWORK_AUTHENTICATION_REQUIRED)
                .body(())
                .unwrap()
        }
    }

}
