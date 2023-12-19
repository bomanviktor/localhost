use crate::server::errors::error;
use crate::server::method::handle_method;
use crate::server::redirections::redirect;
use crate::server::*;

pub fn handle_client(stream: &mut TcpStream, config: &ServerConfig) -> io::Result<()> {
    let mut buffer = [0; 1024];

    let bytes_read = stream.read(&mut buffer)?;

    let request_string = match String::from_utf8(buffer[..bytes_read].to_vec()) {
        Ok(request_str) => request_str,
        Err(e) => {
            eprintln!("Error reading from buffer to string: {e}");
            return Ok(());
        }
    };

    let request = match get_request(config, &request_string) {
        Ok(req) => req,
        Err(e) => return serve_response(stream, error(e, config)),
    };

    let route = match get_route(&request, config) {
        Ok(route) => route,
        Err((code, path)) if code.is_redirection() => {
            return serve_response(stream, redirect(code, config, request.version(), path));
        }
        Err((code, _)) => return serve_response(stream, error(code, config)),
    };

    if is_cgi_request(&request.uri().to_string()) {
        match execute_cgi_script(&request_string, config, &route) {
            Ok(resp) => {
                stream.write_all(&resp).unwrap();
                stream.flush().expect("could not flush");
            }
            Err(code) => return serve_response(stream, error(code, config)),
        }
        return Ok(());
    }

    if request.method().is_safe() {
        match handle_safe_request(&request, config, &route) {
            Ok(response) => serve_response(stream, response)?,
            Err(code) => serve_response(stream, error(code, config))?,
        }
    } else {
        match handle_unsafe_request(&request, config, &route) {
            Ok(response) => serve_response(stream, response)?,
            Err(code) => serve_response(stream, error(code, config))?,
        };
    }
    Ok(())
}

pub fn serve_response(stream: &mut TcpStream, response: Response<Bytes>) -> io::Result<()> {
    unsafe {
        stream.write_all(&format(response))?;
    }

    match stream.flush() {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn handle_safe_request(
    req: &Request<String>,
    config: &ServerConfig,
    route: &Route,
) -> Result<Response<Bytes>, StatusCode> {
    let resp = handle_method(route, req, config).unwrap_or_default();
    Ok(resp)
}

fn handle_unsafe_request(
    req: &Request<String>,
    config: &ServerConfig,
    route: &Route,
) -> Result<Response<Bytes>, StatusCode> {
    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .version(req.version())
        .header(HOST, config.host);

    // Set the Content-Type header or respond with 400 - Bad Request
    if let Some(content_type) = req.headers().get(CONTENT_TYPE) {
        resp = resp.header(CONTENT_TYPE, content_type);
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Set the Content-Length header or respond with 411 - Length Required
    let mut content_length = false;
    if let Some(length) = req.headers().get(CONTENT_LENGTH) {
        resp = resp.header(CONTENT_LENGTH, length);
        content_length = true;
    }

    // Requires a length, has no length, and is a
    if route.length_required && !content_length {
        return Err(StatusCode::LENGTH_REQUIRED);
    }

    // Get the body of the response
    if req.body().len() > config.body_size_limit {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    let body = handle_method(route, req, config).unwrap_or_default();
    Ok(resp.body(body).unwrap())
}
