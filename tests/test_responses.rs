mod mock;

use http::{StatusCode, Version};
use localhost::server::content_type;
use localhost::server::informational::informational;
use localhost::server::redirections::redirect;
use mock::*;
use std::collections::HashMap;
#[test]
fn test_informational() {
    let conf = &mock_server_config();
    let code = StatusCode::from_u16(100).unwrap();
    let resp = informational(code, conf, Version::HTTP_11);
    assert_eq!(
        resp.status(),
        code,
        "Code {} should be {code}",
        resp.status()
    );
}

#[test]
fn test_redirect() {
    let conf = &mock_server_config();
    let code = StatusCode::from_u16(300).unwrap();
    let path = "/here".to_string();
    let resp = redirect(code, conf, Version::HTTP_11, path);
    assert_eq!(
        resp.status(),
        code,
        "Code {} should be {code}",
        resp.status()
    );
}

#[test]
fn test_content_type() {
    let test_cases: HashMap<&str, &str> = [
        ("html", "text/html"),
        ("css", "text/css"),
        ("js", "text/javascript"),
        ("txt", "text/plain"),
        ("xml", "text/xml"),
        ("http", "message/http"),
        ("jpeg", "image/jpeg"),
        ("jpg", "image/jpeg"),
        ("png", "image/png"),
        ("gif", "image/gif"),
        ("bmp", "image/bmp"),
        ("svg", "image/svg+xml"),
        ("aac", "audio/aac"),
        ("eac3", "audio/eac3"),
        ("mp3", "audio/mpeg"),
        ("ogg", "audio/ogg"),
        ("mp4", "video/mp4"),
        ("webm", "video/webm"),
        ("ogv", "video/ogg"),
        ("json", "application/json"),
        ("pdf", "application/pdf"),
        ("zip", "application/zip"),
        ("tar", "application/x-tar"),
        ("gz", "application/gzip"),
        ("exe", "application/octet-stream"),
        ("msi", "application/octet-stream"),
        ("woff", "application/font-woff"),
        ("woff2", "application/font-woff2"),
        ("ttf", "application/font-sfnt"),
        ("otf", "application/font-sfnt"),
        ("unknown", "text/plain"), // Default case
    ]
    .iter()
    .cloned()
    .collect();

    // Iterate over test cases and assert equality
    for (input, expected) in test_cases {
        assert_eq!(content_type(input), expected);
    }
}
