use std::thread;

use reqwest;
use std::time::Duration;

#[tokio::test]
async fn test_chunked_transfer_encoding() {
    // Start your server in a separate thread or use a test setup

    //sleep for 1 second to allow the server to start

    // Construct and send a chunked request

    let body = "4\r\nWiki\r\n5\r\npedia\r\n7\r\n in\r\n\r\nchunks.\r\n0\r\n\r\n";
    println!("Sending request body:\n{}", body);

    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:8080/test")
        .header("Transfer-Encoding", "chunked")
        .header("Content-Type", "text/plain")
        .body(body)
        .send()
        .await
        .unwrap();

    // Check the response status and body
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    // Add more assertions as necessary
    //print the response
    println!("Response: {:?}", response.text().await.unwrap());
}
