use crate::log::*;
use crate::server::errors::error;
use crate::server::handle_method;
use crate::server::redirections::redirect;
use crate::server::*;

pub fn handle_client(stream: &mut TcpStream, config: &ServerConfig) -> io::Result<()> {
    let mut buffer = [0; 1024];

    let bytes_read = stream.read(&mut buffer)?;

    let request_string = match String::from_utf8(buffer[..bytes_read].to_vec()) {
        Ok(request_str) => request_str,
        Err(e) => {
            log(
                LogFileType::Server,
                format!("Error reading from buffer to string: {e}"),
            );
            return Ok(());
        }
    };

    let request = match get_request(config, &request_string) {
        Ok(req) => req,
        Err(e) => {
            log(LogFileType::Server, format!("Error: {}", e));
            return serve_response(stream, error(e, config));
        }
    };

    let route = match get_route(&request, config) {
        Ok(route) => route,
        Err((code, path)) if code.is_redirection() => {
            return serve_response(stream, redirect(code, config, request.version(), path));
        }
        Err((code, _)) => {
            log(LogFileType::Server, format!("Error: {}", &code));
            return serve_response(stream, error(code, config));
        }
    };

    if route.handler.is_some() {
        let handler = route.handler.unwrap();
        match handler(&request, config) {
            Ok(response) => return serve_response(stream, response),
            Err(code) => {
                log(LogFileType::Server, format!("Error: {}", &code));
                return serve_response(stream, error(code, config));
            }
        }
    }

    if route.settings.is_some() && is_cgi_request(&request.uri().to_string()) {
        match execute_cgi_script(&request_string, config, &route.settings.unwrap()) {
            Ok(resp) => {
                stream.write_all(&resp).unwrap();
                stream.flush().expect("could not flush");
            }
            Err(code) => {
                log(LogFileType::Server, format!("Error: {}", &code));
                return serve_response(stream, error(code, config));
            }
        }
        return Ok(());
    }

    match handle_method(&route, &request, config) {
        Ok(response) => serve_response(stream, response)?,
        Err(code) => {
            log(LogFileType::Server, format!("Error: {}", &code));
            serve_response(stream, error(code, config))?
        }
    }
    Ok(())
}

pub fn serve_response(stream: &mut TcpStream, response: Response<Bytes>) -> io::Result<()> {
    unsafe {
        stream.write_all(&format_response(response))?;
    }
    stream.flush()
}
