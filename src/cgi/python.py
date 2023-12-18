#!/usr/bin/env python3

# Print the CGI headers
print("HTTP/1.1 200")
print("Host: 127.0.0.1")
print("Content-type: text/html\n")

# Print the HTML content with "Hello World" and specified font-family
print("<html><head><style>")
print("body {font-family: sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;}")
print("h1 {text-align: center;}")
print("</style></head><body>")
print("<h1>Hello World, from Python CGI!</h1>")
print("</body></html>")
