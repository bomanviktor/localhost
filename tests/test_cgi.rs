mod mock;

use http::StatusCode;
use localhost::server::execute_cgi_script;
use mock::*;

#[test]
fn test_get() {
    let conf = &mock_server_config();
    for path in ["javascript.js", "php.php", "python.py", "ruby.rb"] {
        let req = &mock_request(http::Method::GET, &format!("/cgi/{path}"), None, None);
        assert!(execute_cgi_script(req, conf).is_ok());
    }
}

#[test]
fn test_invalid_route() {
    let conf = &mock_server_config();
    let req = &mock_request(http::Method::GET, "/cgi/test", None, None);

    assert!(execute_cgi_script(req, conf).is_err());
}

#[test]
fn test_no_cgi_def() {
    let conf = &mock_server_config();
    let req = &mock_request(http::Method::GET, "/test.txt", None, None);

    assert!(execute_cgi_script(req, conf).is_err_and(|e| e == StatusCode::BAD_REQUEST));
}

#[test]
fn add_headers_to_request() {
    let conf = &mock_server_config();
    let headers = vec![
        ("Accept", "bytes"),
        ("Content-Length", "123"),
        ("Content-Type", "text/html"),
        ("Accept-Charset", "utf-8"),
        ("Accept-Encoding", "none"),
        ("Accept-Language", "en-us"),
        ("Forwarded", "hell yea"),
        ("Host", "grit:lab special"),
        ("Proxy-Authorization", "Yep"),
        ("User-Agent", "Cia"),
        ("Cookie", "yummy"),
        ("Theodore", "Kaczynski"),
        ("Kek", ""),
    ];
    let req = &mock_request(http::Method::GET, "/cgi/php.php", None, Some(headers));
    assert!(execute_cgi_script(req, conf).is_ok());
}
