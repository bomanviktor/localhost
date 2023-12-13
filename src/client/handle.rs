use crate::server_config::ServerConfig;
use http::{Response, StatusCode};
use std::io::{Read, Write};
use std::net::TcpStream;

pub fn handle_client(mut stream: TcpStream, _config: &ServerConfig) {
    //println!("{config:#?}"); // Just printing the current config
    let mut buffer = [0; 1024];

    // Attempt to read the stream into the buffer
    if let Err(error) = stream.read(&mut buffer) {
        eprintln!("Error reading from stream: {}", error);
        return;
    }

    // Attempt to convert the buffer to a String
    // Implementation is needed to compare against available Uri
    let _request_str = match String::from_utf8(buffer.to_vec()) {
        Ok(request_str) => request_str,
        Err(error) => {
            eprintln!("Error converting buffer to String: {}", error);
            return;
        }
    };

    // Do something based on request type here. Can check for server configs etc.
    /*
    if request.method() != Method::GET {
        eprintln!("Invalid method."); // Will obviously be a function that does this here.
        return;
    }
     */

    // Handle the HTTP request
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body("Hello world")
        .unwrap();

    let response = format!(
        "{:?} {}\nContent-Type: text/plain\n\n{}",
        response.version(),
        response.status(),
        response.body()
    );
    println!("response: {response}");

    if let Err(error) = stream.write_all(response.as_bytes()) {
        eprintln!("Error writing response: {error}");
    }

    stream.flush().expect("could not flush");
    //stream.write(TEMPORARY_RESPONSE.as_bytes()).expect("Something went terribly wrong.");
}
