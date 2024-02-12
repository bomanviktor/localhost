# localhost

_A single threaded HTTP server written in Rust as a part of the grit:lab curriculum._

[Project description](https://github.com/01-edu/public/tree/master/subjects/localhost)

## Features

- Custom handlers
- Standard handlers for `GET, HEAD, OPTIONS, TRACE, POST, PUT, DELETE & PATCH`
- Support for chunked requests with the `Transfer-Encoding` header.
- Support for `JavaScript, Python, PHP and Ruby` CGI. 
- Sessions
- Server logs
- Dynamic default error page

### Quick start guide
1. Install Rust
2. run `cargo run` in the root of this directory

_The demo configuration will give you these following routes:_
- `/api/update-cookie` - _Handler to update a cookie on the server_
- `/api/get-cookie` - _Handler to get a cookie from the server_
- `/api/cookie-demo` - _Dynamic session demo_
- `/cgi` - _Demo path for implemented CGI_
- `/files` - _Access anything you want in the /files directory. Highly recommend to remove this endpoint in production._
- `/test.txt` - _Used for testing files on the server_
- `/test-dir` - _Used for testing directories on the server_
