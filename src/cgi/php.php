<?php
// Print the CGI headers
echo "HTTP/1.1 200\n";
echo "Host: 127.0.0.1\n";
echo "Content-type: text/html\n\n";

// Print the HTML content with "Hello World" and specified font-family
echo "<html><head><style>";
echo "body {font-family: sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;}";
echo "h1 {text-align: center;}";
echo "</style></head><body>";
echo "<h1>Hello World, from PHP CGI!</h1>";
echo "</body></html>";
?>
