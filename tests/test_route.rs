mod mock;
use mock::*;

mod test_get_route {
    use super::*;
    use http::{Method, StatusCode};
    use localhost::server::get_route;
    #[test]
    fn test_valid_route() {
        let req = &mock_request(Method::GET, "/test.txt", None, None);
        let config = &mock_server_config();

        let route = get_route(req, config);

        assert!(route.is_ok_and(|route| route.url_path == "/test.txt"));
    }

    #[test]
    fn test_redirection() {
        let req = &mock_request(Method::GET, "/redirection", None, None);
        let config = &mock_server_config();

        let route = get_route(req, config);

        assert!(route.is_err_and(|(code, path)| {
            code == StatusCode::TEMPORARY_REDIRECT && path == "/tests/redirect.txt"
        }));
    }

    #[test]
    fn test_not_found() {
        let req = &mock_request(Method::PATCH, "/jews", None, None);
        let config = &mock_server_config();
        let route = get_route(req, config);

        assert!(route.is_err_and(|(code, path)| { code == StatusCode::NOT_FOUND && path == "" }));
    }

    #[test]
    fn test_method_not_allowed() {
        let req = &mock_request(Method::PATCH, "/tests/redirect.txt", None, None);
        let config = &mock_server_config();
        let route = get_route(req, config);

        assert!(route
            .is_err_and(|(code, path)| { code == StatusCode::METHOD_NOT_ALLOWED && path == "" }));
    }
}
