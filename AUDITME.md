# Functional

## How does an HTTP server work?

You set up a server that listens on specified ports for incoming connections. If a connection is accepted, then you can start observing this connection for incoming events (HTTP requests) and handle them according to the procotols. Handling such a request might be about sending the client data from the server or inserting data to the server, after the event is handled the server will send a response to the client telling if the request was successful or not.

## Which function was used for I/O Multiplexing and how does it works?

We chose to go with the mio package for it allows nonblocking IO operations.
It works by doing IO operations in chunks, which gives the appearance of concurrency (magical taskswitching)

## Is the server using only one select (or equivalent) to read the client requests and write answers?

We use one single loop to manage all IO operations and use mio poll instead of select.
It observers the sockets for writability/readability and informs the application about the status

## Why is it important to use only one select and how was it achieved?

It's a simple centralized approach and doesn't require multithreading
Achieved by using a single loop

## Read the code that goes from the select (or equivalent) to the read and write of a client, is there only one read or write per client per select (or equivalent)?

Yupyup

## Are the return values for I/O functions checked properly?

Mmmyeesss

## If an error is returned by the previous functions on a socket, is the client removed?

If the error is of the blocking kind, we keep the connection and move on to the next iteration in our management loop.
If the error is something else, we consider it non-recoverable and deregister the client.

## Is writing and reading ALWAYS done through a select (or equivalent)?

Yes due to the single IO operations loop

# Configuration file

Check the configuration file and modify it if necessary. Are the following configurations working properly:

## Setup a single server with a single port.

In config.rs / server_config make sure that only one ServerConfig is added to the vector

```
host: "127.0.0.1",
ports: vec![8080],
```

## Setup multiple servers with different port.

Add multiple ServerConfig instances in to the vector, another server could be for example:

```
host: "0.0.0.0",
ports: vec![8081]
```

## Setup multiple servers with different hostnames (for example: curl --resolve test.com:80:127.0.0.1 http://test.com/).

Hostnames have to be paired with IPs in the machines filesystem before they can be assigned in the ServerConfig

This aims to confirm if your server correctly distinguishes between requests for different hostnames even though they resolve to the same IP and port.

## Setup custom error pages.

Works as long as no tailing slash is appended to the path

## Limit the client body (for example: curl -X POST -H "Content-Type: plain/text" --data "BODY with something shorter or longer than body limit").

Returns payload too large in case you hit the limit.
You can simulate this by changing body_size_limit in the serverconfig for example.

## Setup routes and ensure they are taken into account.

## Setup a default file in case the path is a directory.

Example route:

```
            Route {
                url_path: "/files/mega-dir",
                methods: vec![http::Method::GET],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: None,
                    default_if_url_is_dir: Some("/files/dir.html"),
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            }
```

## Setup a list of accepted methods for a route (for example: try to DELETE something with and without permission).

# Methods & cookies

## For each method be sure to check the status code (200, 404 etc):

Works

## Are the GET requests working properly?

Works, you can for example:

Run the server and navigate to http://localhost:8080/files/

You can also do this via terminal:
curl -i http://localhost:8080/files/

...or use Postman

## Are the POST requests working properly?

Send a test file:

```
curl -X POST -H "Content-Type: text/plain" -i  --data-binary @yourfile.txthttp://localhost:8080/files/filenameOnServer.txt
```

Content-Type and file extension have to be set according the the type of file you are about to send

Run

```
curl -i  http://localhost:8080/files/filenameOnServer.txt
```

to verify data integrity

...or use postman

## Are the DELETE requests working properly?

```
curl -i -X DELETE http://localhost:8080/files/filenameOnServer.txt
```

You can try to GET the resource again to verify deletion if statuscode doesn't satisfy your high standards

## Test a WRONG request, is the server still working properly?

```
curl -X KEKTUS http://localhost:8080/files/
curl http://localhost:8080/%zz%zz%zz
```

## Upload some files to the server and get them back to test they were not corrupted.

POST & GET your favourite .mp3 for example, or just a txt file

## A working session and cookies system is present on the server?

There is a cookie demo API

# Interaction with the browser

## Is the browser connecting with the server with no issues?

## Are the request and response headers correct? (It should serve a full static website without any problem).

## Try a wrong URL on the server, is it handled properly?

## Try to list a directory, is it handled properly?

## Try a redirected URL, is it handled properly?

Navigate to http://127.0.0.1:8080/redirection-test

## Check the implemented CGI, does it works properly with chunked and unchunked data?

You can request a response to be sent in chunks by adding the Transfer-Encoding header in a get request for example.

This header will be found in the response also, if one wishes to see the individual chunks, you could use Wireshark or other software.

But keeping it simple:

```
curl -i -H "Transfer-Encoding: chunked" http://localhost:8080/cgi/python.py
```

# Port issues

## Configure multiple ports and websites and ensure it is working as expected.

## Configure the same port multiple times. The server should find the error.

```
ports: vec![8080, 8080]
```

## Configure multiple servers at the same time with different configurations but with common ports. Ask why the server should work if one of the configurations isn't working.

This aims to validate your server's configuration handling. You'll configure multiple servers simultaneously, each with different configurations but with shared ports. If one of these configurations isn't valid or encounters an issue, your server should continue to function for the other configurations without being entirely disrupted. The purpose is to ensure that an error in one server's configuration doesn't bring down the entire server if other configurations are correctly set up.

# Siege & stress test

## Use siege with a GET method on an empty page

Availability should be at least 99.5% with the command siege -b [IP]:[PORT].

## Check if there is no memory leak (you could use some tools like top).

## Check if there is no hanging connection.
