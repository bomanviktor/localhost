use crate::mock::{mock_request, mock_server_config};
use http::header::{COOKIE, SET_COOKIE};
use http::Method;
use localhost::server::{cookie_demo, update_cookie, validate_cookie};

mod mock;
#[test]
fn test_update_cookie() {
    let conf = &mock_server_config();

    let req = &mock_request(Method::POST, "", None, None);
    let resp = update_cookie(req, conf).unwrap();
    assert!(resp.headers().contains_key(SET_COOKIE));

    let req = &mock_request(
        Method::POST,
        "",
        None,
        Some(vec![("cookie", "grit:lab=cookie")]),
    );

    let resp = update_cookie(req, conf).unwrap();
    assert!(resp.headers().get(SET_COOKIE).is_some_and(|header| header
        .to_str()
        .is_ok_and(|cookie| cookie.contains("path=/;"))));
}

#[test]
fn test_cookie_demo() {
    let conf = &mock_server_config();
    let req = &mock_request(
        Method::POST,
        "",
        None,
        Some(vec![("Transfer-Encoding", "Chunked")]),
    );
    let resp = cookie_demo(req, conf).unwrap();
    assert!(resp.status().is_success());
}

#[test]
fn test_validate_cookie() {
    let conf = &mock_server_config();
    let req = &mock_request(
        Method::POST,
        "",
        None,
        Some(vec![(COOKIE.as_str(), "grit:lab=cookie")]),
    );
    let resp = validate_cookie(req, conf).unwrap();

    assert!(resp.headers().contains_key(COOKIE));
}
