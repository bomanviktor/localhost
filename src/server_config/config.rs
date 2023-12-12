use crate::server_config::route::cgi::Cgi;
pub use crate::server_config::*;
use std::collections::HashMap;

impl<'a> ServerConfig<'a> {
    pub fn set() -> Vec<ServerConfig<'a>> {
        vec![ServerConfig {
            host: "kek",
            ports: vec![1000, 2000],
            default_error_paths: vec![
                Path::new("/400.html"),
                Path::new("/401.html"),
                Path::new("/404.html"),
                Path::new("/405.html"),
                Path::new("/500.html"),
            ],
            body_size_limit: 5000,
            routes: vec![Route {
                accepted_http_methods: vec![http::Method::GET],
                http_redirections: HashMap::from([
                    ("/this", "/is"),
                    ("/how", "/to"),
                    ("/redirect", "/http"),
                ]),
                default_if_url_is_dir: Path::new("some default"),
                default_if_request_is_dir: Path::new("some other default"),
                cgi: Cgi::PHP,
                list_directory: true,
            }],
        }]

        /*

        let mut server_configs = Vec::new();
              let config = ConfigBuilder::new()
                  .host("test")
                  .ports(vec![1000, 2000])
                  .default_error_paths(vec![
                      Path::new("/400.html"),
                      Path::new("/401.html"),
                      Path::new("/404.html"),
                      Path::new("/405.html"),
                      Path::new("/500.html"),
                  ])
                  .body_size_limit(5000) // Bytes
                  .routes( vec![Route {
                      accepted_http_methods: vec![http::Method::GET],
                      http_redirections: HashMap::from([
                          ("/this", "/is"),
                          ("/how", "/to"),
                          ("/redirect", "/http"),
                      ]),
                      default_if_url_is_dir: Path::new("some default"),
                      default_if_request_is_dir: Path::new("some other default"),
                      cgi: Cgi::PHP,
                      list_directory: true,
                  }]) // Insert routes here
                  .build();
              server_configs.push(config);

              server_configs
                  */
    }
}
