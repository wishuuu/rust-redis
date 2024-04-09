use anyhow::Result;
use db::DataLayer;
use resp::{RespHandler, Value};
use tokio::net::{TcpListener, TcpStream};

mod resp;
mod db;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let data_layer = DataLayer::new();

    loop {
        let stream = listener.accept().await;
        let data_layer = data_layer.clone();
        match stream {
            Ok((stream, _)) => {
                println!("accepted new connection");
                tokio::spawn(async move {
                    handle_request(stream, data_layer).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn handle_request(stream: TcpStream, db: DataLayer) {
    let mut handler = RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();

        println!("Got value: {:?}", value);

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            match command.to_ascii_uppercase().as_str() {
                "PING" => Value::SimpleString("PONG".to_string()),
                "ECHO" => args.first().unwrap().clone(),
                "SET" => db.clone().set_value(args.first().unwrap().clone(), args[1].clone()),
                "GET" => db.clone().get_value(args.first().unwrap().clone()),
                c => panic!("Cannot handle commad {}", c),
            }
        } else {
            break;
        };

        println!("Sending response: {:?}", response);

        handler.write_value(response).await.unwrap();
    }
}

fn extract_command(value: Value) -> Result<(String, Vec<Value>)> {
    match value {
        Value::Array(a) => Ok((
            unpack_bulk_str(a.first().unwrap().clone())?,
            a.into_iter().skip(1).collect(),
        )),
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}

fn unpack_bulk_str(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Expected command to be bulk string")),
    }
}
