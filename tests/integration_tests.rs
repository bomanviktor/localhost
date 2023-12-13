mod common;
mod test_config {
    use localhost::server_config::server_config;
    #[test]
    fn test_fields() {
        let configs = server_config();
        for c in configs {
            assert!(!c.host.is_empty());
            assert!(!c.ports.is_empty());
            assert!(!c.default_error_paths.is_empty());
            assert!(c.body_size_limit > 0);
            assert!(!c.routes.is_empty());
        }
    }
}
