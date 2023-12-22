// Print the CGI headers
console.log("HTTP/1.1 200");
console.log("Host: 127.0.0.1");
console.log("Set-Cookie: test=test-cookie");
console.log("Content-type: text/html\n");

// Print the HTML content with "Hello World" and specified font-family
console.log("<html><head><style>");
console.log("body {font-family: sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;}");
console.log("h1 {text-align: center;}");
console.log("</style></head><body>");
console.log("<h1>Hello World, from JavaScript CGI!</h1>");
console.log("</body></html>");
