#!/usr/bin/env ruby
# Print the HTML content with "Hello World" and specified font-family
puts "<html><head><style>"
puts "body {font-family: sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;}"
puts "h1 {text-align: center;}"
puts "</style></head><body>"
puts "<h1>Hello World, from Ruby CGI!</h1>"
puts "</body></html>"
