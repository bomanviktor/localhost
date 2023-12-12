use crate::server_config::ServerConfig;
use http::{Method, Request, Response, StatusCode};
use std::io::{Read, Write};
use std::net::TcpStream;

const TEMPORARY_RESPONSE: &str = "
Somebody once told me
The world is gonna roll me
I ain't the sharpest tool in the shed
She was looking kind of dumb
With her finger and her thumb
In the shape of an L on her forehead

Well, the years start coming
And they don't stop coming
Fed to the rules and I hit the ground running
Didn't make sense not to live for fun
Your brain gets smart
But your head gets dumb

So much to do, so much to see
So what's wrong with taking the back streets?
You'll never know if you don't go
You'll never shine if you don't glow

Hey now, you're an all star
Get your game on, go play
Hey now, you're a rock star
Get the show on, get paid
And all that glitters is gold
Only shooting stars break the mold

It's a cool place and they say it gets colder
You're bundled up now
But wait till you get older
But the meteor men beg to differ
Judging by the hole in the satellite picture

The ice we skate is getting pretty thin
The water is getting warm
So you might as well swim
My world's on fire, how about yours?
That's the way I like it
And I never get bored

Hey now, you're an all star
Get your game on, go play
Hey now, you're a rock star
Get the show on, get paid
And all that glitters is gold
Only shooting stars break the mold

Hey now, you're an all star
Get your game on, go play
Hey now, you're a rock star
Get the show on, get paid
And all that glitters is gold
Only shooting stars

Somebody once asked
Could I spare some change for gas?
I need to get myself away from this place
I said: Yep, what a concept
I could use a little fuel myself
And we could all use a little change

Well the years start coming
And they don't stop coming
Fed to the rules and I hit the ground running
Didn't make sense not to live for fun
Your brain gets smart
But your head gets dumb

So much to do, so much to see
So what's wrong with taking the back streets
You'll never know if you don't go (go!)
You'll never shine if you don't glow

Hey now, you're an all star
Get your game on, go play
Hey now, you're a rock star
Get the show on, get paid

And all that glitters is gold
Only shooting stars break the mold
And all that glitters is gold
Only shooting stars break the mold";

pub fn handle_client(mut stream: TcpStream, _config: &ServerConfig) {
    //println!("{config:#?}"); // Just printing the current config
    let mut buffer = [0; 1024];

    // Attempt to read the stream into the buffer
    if let Err(error) = stream.read(&mut buffer) {
        eprintln!("Error reading from stream: {}", error);
        return;
    }

    // Attempt to convert the buffer to a String
    // Implementation is needed to compare against available Uri
    /*
    let request_str = match String::from_utf8(buffer.to_vec()) {
        Ok(request_str) => request_str,
        Err(error) => {
            eprintln!("Error converting buffer to String: {}", error);
            return;
        }
    };

     */

    // Attempt to parse the HTTP request
    let request = match Request::builder().method("GET").body(()) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("Error creating request: {}", error);
            return;
        }
    };

    // Do something based on request type here. Can check for server configs etc.
    if request.method() != Method::GET {
        eprintln!("Invalid method."); // Will obviously be a function that does this here.
        return;
    }

    // Handle the HTTP request
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body(TEMPORARY_RESPONSE.as_bytes())
        .unwrap();

    if let Err(error) = stream.write_all(response.body()) {
        eprintln!("Error writing response: {error}");
    }
    //stream.write(TEMPORARY_RESPONSE.as_bytes()).expect("Something went terribly wrong.");
}
