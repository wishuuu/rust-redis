use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn handle_request(request: &str, stream: &mut TcpStream) -> Result<(), String> {
    match request {
        "ping" => {
            let response = b"+PONG\r\n";
            match stream.write_all(response) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        }
        _ => Err(format!("Do not support request {}", request)),
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let mut request = String::new();
                _stream.read_to_string(&mut request).unwrap();

                request.split("\r\n").for_each(|req| {
                    match handle_request(&req, &mut _stream) {
                        Ok(_) => (),
                        Err(e) => println!("error: {}", e),
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
