#!/usr/bin/env python3
import random

# Function to generate a line of ASCII art
def generate_ascii_art_line():
    chars = ['*', '#', '@', '&', '%', '$']
    return ''.join(random.choice(chars) for _ in range(80))

# Start of the HTML content
print("<html><head><style>")
print("body {font-family: monospace; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; overflow: auto;}")
print("pre {text-align: left;}")
print("</style></head><body>")
print("<pre>")

# Generate and print 1000 lines of ASCII art
for _ in range(1000):
    print(generate_ascii_art_line())

# End of the HTML content
print("</pre>")
print("</body></html>")
