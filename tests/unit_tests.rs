use std::fs;

use http::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    Method, Request, StatusCode, Uri,
};
use localhost::{
    server::{content_type, get_method, handle_method, method_is_allowed},
    server_config::{
        route::{Route, Settings},
        ServerConfig,
    },
};

#[test]
fn test_get_method_valid() {
    let request_line = "GET /index.html HTTP/1.1";
    assert_eq!(get_method(request_line).unwrap(), Method::GET);
}

#[test]
fn test_get_method_empty() {
    let request_line = "";
    assert!(get_method(request_line).is_err());
}

#[test]
fn test_get_method_no_method() {
    let request_line = "/index.html HTTP/1.1";
    assert!(get_method(request_line).is_err());
}

#[test]
fn test_method_is_allowed() {
    // Create a Route with the allowed methods and dummy values
    let route = Route {
        methods: vec![
            Method::GET,
            Method::OPTIONS,
            Method::HEAD,
            Method::TRACE,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            // Method::CONNECT, // Excluded as it's unimplemented
        ],
        url_path: "/",
        handler: None,
        settings: None,
    };

    // Test each method to ensure it's allowed
    assert!(method_is_allowed(&Method::GET, &route));
    assert!(method_is_allowed(&Method::OPTIONS, &route));
    assert!(method_is_allowed(&Method::HEAD, &route));
    assert!(method_is_allowed(&Method::TRACE, &route));
    assert!(method_is_allowed(&Method::POST, &route));
    assert!(method_is_allowed(&Method::PUT, &route));
    assert!(method_is_allowed(&Method::PATCH, &route));
    assert!(method_is_allowed(&Method::DELETE, &route));
    // assert!(method_is_allowed(&Method::CONNECT, &route)); // Excluded as it's unimplemented

    // Test with a method that is not allowed
    assert!(!method_is_allowed(
        &Method::from_bytes(b"UNKNOWN").unwrap(),
        &route
    ));
}

#[test]
fn test_handle_method_get() {
    let route = mock_route();
    let request = mock_request(Method::GET);
    let config = mock_server_config();

    let result = handle_method(&route, &request, &config);
    assert!(result.is_ok());
    // Additional assertions based on the expected response
}

#[test]
fn test_handle_method_post() {
    let route = mock_route();
    let request = mock_request(Method::POST);
    let config = mock_server_config();

    let result = handle_method(&route, &request, &config);
    assert!(result.is_ok());
    // Additional assertions based on the expected response
}

// Additional tests for OPTIONS, HEAD, TRACE, PUT, PATCH, DELETE

#[test]
fn test_handle_method_unsupported() {
    let route = mock_route();
    let request = mock_request(Method::from_bytes(b"UNSUPPORTED").unwrap());
    let config = mock_server_config();

    let result = handle_method(&route, &request, &config);
    assert!(matches!(result, Err(StatusCode::BAD_REQUEST)));
}

#[test]
fn test_handle_method_head() {
    let route = mock_route();
    let request = mock_request(Method::HEAD);
    let config = mock_server_config();

    let result = handle_method(&route, &request, &config);
    assert!(result.is_ok());
    let response = result.unwrap();
    // Check that the response body is empty
    assert!(response.body().is_empty());
    //Check that the response headers are correct
    assert!(response.headers().contains_key("Content-Length"));
    assert!(response.headers().contains_key("Content-Type"));
}

#[test]
fn test_handle_method_options() {
    let route = mock_route();
    let request = mock_request(Method::OPTIONS);
    let config = mock_server_config();

    let result = handle_method(&route, &request, &config);
    assert!(result.is_ok());
    let response = result.unwrap();
    // Check that we receive the correct methods that are available
    let allowed_methods = response.headers().get("Allow").unwrap().to_str().unwrap();
    assert!(
        allowed_methods.contains("GET")
            && allowed_methods.contains("POST")
            && allowed_methods.contains("HEAD")
            && allowed_methods.contains("OPTIONS")
            && allowed_methods.contains("TRACE")
            && allowed_methods.contains("PUT")
            && allowed_methods.contains("PATCH")
            && allowed_methods.contains("DELETE")
    );
}

#[test]
fn test_handle_method_trace() {
    let route = mock_route();
    let mut request = mock_request(Method::TRACE);
    request
        .headers_mut()
        .insert("Max-Forwards", "10".parse().unwrap());
    // Sensitive headers are added but will be excluded in the response
    request
        .headers_mut()
        .insert("Cookie", "test_cookie".parse().unwrap());
    request
        .headers_mut()
        .insert("Authorization", "Basic dGVzdDp0ZXN0".parse().unwrap());

    let config = mock_server_config();

    let result = handle_method(&route, &request, &config);
    assert!(result.is_ok());
    let response = result.unwrap();

    // Use debug representation to reconstruct the request string
    let request_representation = format!("{:?}\r\n", request)
        .lines()
        .filter(|line| !line.contains("Cookie:") && !line.contains("Authorization:"))
        .collect::<Vec<_>>()
        .join("\r\n");

    // Convert response body to String for comparison
    let response_body_str = String::from_utf8(response.body().clone())
        .expect("Failed to convert response body to String");

    // Check that the response body matches the reconstructed request
    assert_eq!(response_body_str, request_representation);
}

#[test]
fn test_handle_method_put() {
    let route = mock_route();
    let mut request = mock_request(Method::PUT);
    let config = mock_server_config();

    // Set up a test file path and body content
    let test_file_path = "/test_put.txt";
    let test_body_content = "Test PUT content kek";
    // Construct a new Uri with the test file path
    let new_uri = format!("http://localhost:8080{}", test_file_path)
        .parse::<Uri>()
        .expect("Failed to parse URI");

    request.uri_mut().clone_from(&new_uri);
    *request.body_mut() = test_body_content.to_string();

    // Execute the PUT request
    let result = handle_method(&route, &request, &config);
    assert!(result.is_ok());
    let response = result.unwrap();

    // Check response status
    assert_eq!(response.status(), StatusCode::OK);

    // Check response headers
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        &content_type(test_file_path)
    );
    assert_eq!(
        response.headers().get(CONTENT_LENGTH).unwrap(),
        &test_body_content.len().to_string()
    );

    // Check response body
    assert_eq!(response.body(), &test_body_content.as_bytes().to_vec());

    // Verify that the file was created and contains the correct content
    let file_path = format!("./files{}", test_file_path);
    let cloned_file_path = file_path.clone();
    let file_content = fs::read_to_string(file_path).expect("Failed to read file");
    assert_eq!(file_content, test_body_content);

    // Clean up: remove the test file
    fs::remove_file(cloned_file_path).expect("Failed to remove test file");
}

#[test]
fn test_handle_method_patch() {
    let route = mock_route();
    let config = mock_server_config();

    // Step 1: Create a test file using PUT
    let test_file_path = "/patch_test.txt";
    let initial_content = "Initial Content";
    let modified_content = "Modified Content";
    let mut put_request = mock_request(Method::PUT);
    let put_uri = format!("http://localhost:8080{}", test_file_path)
        .parse::<Uri>()
        .expect("Failed to parse URI");
    put_request.uri_mut().clone_from(&put_uri);
    *put_request.body_mut() = initial_content.to_string();
    let put_result = handle_method(&route, &put_request, &config);
    assert!(put_result.is_ok());

    // Step 2: Modify the file content using PATCH
    let mut patch_request = mock_request(Method::PATCH);
    patch_request.uri_mut().clone_from(&put_uri);
    *patch_request.body_mut() = modified_content.to_string();
    let patch_result = handle_method(&route, &patch_request, &config);
    assert!(patch_result.is_ok());

    // Step 3: Retrieve the modified file content using GET
    let mut get_request = mock_request(Method::GET);
    get_request.uri_mut().clone_from(&put_uri);
    let get_result = handle_method(&route, &get_request, &config);
    assert!(get_result.is_ok());
    let get_response = get_result.unwrap();

    // Check the content of the file
    let response_body_str = String::from_utf8(get_response.body().clone())
        .expect("Failed to convert response body to String");
    assert_eq!(response_body_str, modified_content);

    // Clean up: remove the test file
    let file_path = format!("./files{}", test_file_path);
    fs::remove_file(file_path).expect("Failed to remove test file");
}

#[test]
fn test_handle_method_delete_existing_file() {
    let route = mock_route();
    let config = mock_server_config();

    // Create a file using PUT
    let test_file_path = "/delete_test.txt";
    let test_content = "Test Content";
    let mut put_request = mock_request(Method::PUT);
    put_request
        .uri_mut()
        .clone_from(&Uri::try_from(test_file_path).unwrap());
    *put_request.body_mut() = test_content.to_string();
    assert!(handle_method(&route, &put_request, &config).is_ok());

    // Retrieve the file using GET
    let mut get_request = mock_request(Method::GET);
    get_request
        .uri_mut()
        .clone_from(&Uri::try_from(test_file_path).unwrap());
    assert!(handle_method(&route, &get_request, &config).is_ok());

    // Delete the file using DELETE
    let mut delete_request = mock_request(Method::DELETE);
    delete_request
        .uri_mut()
        .clone_from(&Uri::try_from(test_file_path).unwrap());
    assert!(handle_method(&route, &delete_request, &config).is_ok());

    // Attempt to retrieve the file again using GET
    let mut get_request_again = mock_request(Method::GET);
    get_request_again
        .uri_mut()
        .clone_from(&Uri::try_from(test_file_path).unwrap());
    assert!(matches!(
        handle_method(&route, &get_request_again, &config),
        Err(StatusCode::NOT_FOUND)
    ));
}

#[test]
fn test_handle_method_delete_non_existing_file() {
    let route = mock_route();
    let config = mock_server_config();

    let test_file_path = "/non_existing_file.txt";
    let mut delete_request = mock_request(Method::DELETE);
    delete_request
        .uri_mut()
        .clone_from(&Uri::try_from(test_file_path).unwrap());

    let result = handle_method(&route, &delete_request, &config);
    println!("Delete result: {:?}", result); // Log the result for debugging

    assert!(matches!(result, Err(StatusCode::NOT_FOUND)));
}

// Mock functions and data for testing
fn mock_route() -> Route<'static> {
    let route = Route {
        methods: vec![
            Method::GET,
            Method::OPTIONS,
            Method::HEAD,
            Method::TRACE,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            // Method::CONNECT, // Excluded as it's unimplemented
        ],
        url_path: "/",
        handler: None,
        settings: None,
    };
    route
}

fn mock_request(method: Method) -> Request<String> {
    let request = Request::builder()
        .method(method)
        .uri("http://localhost:8080/test.txt")
        .body(String::new())
        .unwrap();
    request
}

fn mock_server_config() -> ServerConfig<'static> {
    let config = ServerConfig {
        host: "127.0.0.1",
        ports: vec![8080],
        default_error_path: None,
        body_size_limit: 10024,
        routes: vec![
            Route {
                url_path: "/test.txt",
                methods: vec![
                    http::Method::GET,
                    http::Method::POST,
                    http::Method::HEAD,
                    http::Method::OPTIONS,
                    http::Method::TRACE,
                    http::Method::PUT,
                    http::Method::PATCH,
                    http::Method::DELETE,
                ],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: Some("/files"),
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/test_put.txt",
                methods: vec![
                    http::Method::GET,
                    http::Method::POST,
                    http::Method::HEAD,
                    http::Method::OPTIONS,
                    http::Method::TRACE,
                    http::Method::PUT,
                    http::Method::PATCH,
                    http::Method::DELETE,
                ],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: Some("/files"),
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/patch_test.txt",
                methods: vec![
                    http::Method::GET,
                    http::Method::POST,
                    http::Method::HEAD,
                    http::Method::OPTIONS,
                    http::Method::TRACE,
                    http::Method::PUT,
                    http::Method::PATCH,
                    http::Method::DELETE,
                ],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: Some("/files"),
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/delete_test.txt",
                methods: vec![
                    http::Method::GET,
                    http::Method::POST,
                    http::Method::HEAD,
                    http::Method::OPTIONS,
                    http::Method::TRACE,
                    http::Method::PUT,
                    http::Method::PATCH,
                    http::Method::DELETE,
                ],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: Some("/files"),
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/non_existing_file.txt",
                methods: vec![
                    http::Method::GET,
                    http::Method::POST,
                    http::Method::HEAD,
                    http::Method::OPTIONS,
                    http::Method::TRACE,
                    http::Method::PUT,
                    http::Method::PATCH,
                    http::Method::DELETE,
                ],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: Some("/files"),
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
        ],
    };
    config
}
