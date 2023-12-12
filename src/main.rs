use http::status::*;
use localhost::server_config::ServerConfig;

fn main() {
    let configs = ServerConfig::set();
    println!("{:#?}", configs);

    let ok = StatusCode::from_u16(200).unwrap_or_default();
    println!("{ok}");
}
