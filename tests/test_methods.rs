use std::fs;
mod mock;
use mock::*;

use http::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    Method, StatusCode,
};

use localhost::server::{content_type, get_method, handle_method, method_is_allowed};
mod test_misc {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    #[test]
    fn test_method_is_allowed() {
        let route = mock_route();
        // Iterate through all the methods, and assert that they are allowed.
        for method in &route.methods {
            assert!(method_is_allowed(method, &route));
        }

        // 10 random invalid methods
        for _ in 0..10 {
            let s: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(20)
                .map(char::from)
                .collect();

            assert!(!method_is_allowed(
                &Method::from_bytes(s.as_bytes()).unwrap(),
                &route
            ));
        }
    }

    #[test]
    fn test_handle_method_unsupported() {
        let route = mock_route();
        let request = mock_request(
            Method::from_bytes(b"UNSUPPORTED").unwrap(),
            "/test.txt",
            None,
            None,
        );
        let config = mock_server_config();

        let result = handle_method(&route, &request, &config);
        assert!(matches!(result, Err(StatusCode::NOT_IMPLEMENTED)));
    }

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
}

mod test_get {
    use super::*;
    #[test]
    fn test_handle_method_get() {
        let route = mock_route();
        let config = mock_server_config();
        let request = mock_request(Method::GET, "/test.txt", None, None);

        let result = handle_method(&route, &request, &config);
        assert!(result.is_ok());
        // Additional assertions based on the expected response
    }
}

mod test_post {
    use crate::{mock_request, mock_route, mock_server_config};
    use http::Method;
    use localhost::server::handle_method;

    #[test]
    fn test_handle_method_post() {
        let route = mock_route();
        let config = mock_server_config();
        let request = mock_request(
            Method::POST,
            "/test.txt",
            Some("Ehm... missing post tests here"),
            None,
        );

        let result = handle_method(&route, &request, &config);
        assert!(result.is_ok());
        // Additional assertions based on the expected response

        // Missing removing of test files here
    }
}

mod test_head {
    use super::*;
    #[test]
    fn test_handle_method_head() {
        let route = mock_route();
        let request = mock_request(Method::HEAD, "/test.txt", None, None);
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
}

mod test_options {
    use super::*;
    #[test]
    fn test_handle_method_options() {
        let route = mock_route();
        let config = mock_server_config();
        let request = mock_request(Method::OPTIONS, "/test.txt", None, None);

        let result = handle_method(&route, &request, &config);
        assert!(result.is_ok());
        let response = result.unwrap();
        // Check that we receive the correct methods that are available
        let allowed_methods = response.headers().get("Allow").unwrap().to_str().unwrap();
        for method in &route.methods {
            assert!(allowed_methods.contains(&method.to_string().to_ascii_uppercase()))
        }
    }
}

mod test_trace {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn test_handle_method_trace() {
        let config = mock_server_config();
        let route = mock_route();

        let mut request = mock_request(Method::TRACE, "/test.txt", None, None);
        let headers = HashMap::from([
            ("Max-Forwards", "10"),
            ("Cookie", "test_cookie"),
            ("Authorization", "Basic dGVzdDp0ZXN"),
        ]);
        for (key, value) in headers {
            request.headers_mut().insert(key, value.parse().unwrap());
        }

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
}

#[test]
fn test_handle_method_put() {
    let route = mock_route();
    let config = mock_server_config();

    // Set up a test file path and body content
    let test_file_path = "/test_put.txt";
    let test_body_content = "Test PUT content kek";
    // Construct a new Uri with the test file path

    let request = mock_request(Method::PUT, test_file_path, Some(test_body_content), None);

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

mod test_patch {
    use super::*;
    use localhost::type_aliases::Bytes;
    #[test]
    fn test_handle_method_patch() {
        let route = mock_route();
        let config = mock_server_config();

        // Step 1: Create a test file using PUT
        let test_file_path = "/patch_test.txt";
        let initial_content = "Initial Content";
        let modified_content = "Modified Content";
        let put_request = mock_request(Method::PUT, test_file_path, Some(initial_content), None);
        let put_result = handle_method(&route, &put_request, &config);
        assert!(put_result.is_ok());

        // Step 2: Modify the file content using PATCH
        let patch_request =
            mock_request(Method::PATCH, test_file_path, Some(modified_content), None);
        let body = match handle_method(&route, &patch_request, &config) {
            Ok(resp) => resp.body().clone(),
            _ => panic!(),
        };
        // Assert that the content is now updated
        assert_eq!(Bytes::from(modified_content), body);
        // Clean up: remove the test file
        let file_path = format!("./files{}", test_file_path);
        fs::remove_file(file_path).expect("Failed to remove test file");
    }
}

mod test_delete {}
#[test]
fn test_handle_method_delete_existing_file() {
    let route = mock_route();
    let config = mock_server_config();

    // Create a file using PUT
    let test_file_path = "/delete_test.txt";
    let test_content = "Test Content";
    let put_request = mock_request(Method::PUT, test_file_path, Some(test_content), None);
    assert!(handle_method(&route, &put_request, &config).is_ok());

    // Retrieve the file using GET
    let get_request = mock_request(Method::GET, test_file_path, None, None);
    assert!(handle_method(&route, &get_request, &config).is_ok());

    // Delete the file using DELETE
    let delete_request = mock_request(Method::DELETE, test_file_path, None, None);
    assert!(handle_method(&route, &delete_request, &config).is_ok());

    // Attempt to retrieve the file again using GET
    let get_request_again = mock_request(Method::GET, test_file_path, None, None);
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
    let delete_request = mock_request(Method::DELETE, test_file_path, None, None);
    let result = handle_method(&route, &delete_request, &config);
    assert!(matches!(result, Err(StatusCode::NOT_FOUND)));
}
