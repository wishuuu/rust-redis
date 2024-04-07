use std::{
    io::Write,
    net::TcpListener,
};

// fn handle_request(request: &str, stream: &mut TcpStream) -> Result<(), String> {
//    match request {
//        "PING" => {
//            let response = b"+PONG\r\n";
//            match stream.write_all(response) {
//                Ok(_) => Ok(()),
//                Err(e) => Err(e.to_string()),
//            }
//        }
//        _ => Err(format!("Do not support request {}", request)),
//    }
//}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                //let mut request = String::new();
                //_stream.read_to_string(&mut request).unwrap();

                //match handle_request(&request, &mut _stream) {
                //    Ok(_) => (),
                //    Err(e) => println!("error: {}", e)
                //}
                
                let response = b"+PONG\r\n";
                _stream.write_all(response).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
