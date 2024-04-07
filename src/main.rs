use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

// fn handle_request(request: &str, stream: &mut TcpStream) -> Result<(), String> {
//     match request {
//         "ping" => {
//             let response = b"+PONG\r\n";
//             match stream.write_all(response) {
//                 Ok(_) => Ok(()),
//                 Err(e) => Err(e.to_string()),
//             }
//         }
//         _ => Err(format!("Do not support request {}", request)),
//     }
// }

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((mut stream, _)) => {
                println!("accepted new connection");
                tokio::spawn(async move {
                    let mut buf = [0; 512];
                    loop {
                        let read_count = stream.read(&mut buf).await.unwrap();
                        if read_count == 0 {
                            break;
                        }
                        let response = b"+PONG\r\n";
                        stream.write_all(response).await.unwrap();
                    }
                });
            }

            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
