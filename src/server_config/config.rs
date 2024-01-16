//use crate::server::{update_cookie, validate_cookie, Cgi, Server};
use crate::server_config::*;
use std::fs;

pub fn load_config() -> Vec<ServerConfig> {
    let config_str = fs::read_to_string("config.toml").expect("Failed to read config.toml");
    let app_config: AppConfig = toml::from_str(&config_str).expect("Failed to parse config.toml");

    app_config.servers
}
