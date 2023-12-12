use crate::server_config::builder::ConfigBuilder;
pub use crate::server_config::*;

impl<'a> ServerConfig<'a> {
    pub fn set() -> Vec<ServerConfig<'a>> {
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
            .routes(vec![]) // Insert routes here
            .build();
        server_configs.push(config);

        server_configs
    }
}
