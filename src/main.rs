use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use bytes::Bytes;
use resp::{RespHandler, Value};
use tokio::net::{TcpListener, TcpStream};

mod resp;

type Db = Arc<Mutex<HashMap<String, Value>>>;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let stream = listener.accept().await;
        let db = db.clone();
        match stream {
            Ok((stream, _)) => {
                println!("accepted new connection");
                tokio::spawn(async move {
                    handle_request(stream, db).await;
                });
            }

            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn handle_request(stream: TcpStream, db: Db) {
    let mut handler = RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();

        println!("Got value: {:?}", value);

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            match command.to_ascii_uppercase().as_str() {
                "PING" => Value::SimpleString("PONG".to_string()),
                "ECHO" => args.first().unwrap().clone(),
                "SET" => set_value(args.first().unwrap().clone(), args[1].clone(), db.clone()),
                "GET" => get_value(args.first().unwrap().clone(), db.clone()),
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

fn set_value(key: Value, value: Value, db: Db) -> Value {
    let mut db = db.lock().unwrap();
    db.insert(key.clone().serialize(), value.clone());
    Value::SimpleString("OK".to_string())
}

fn get_value(key: Value, db: Db) -> Value {
    let db = db.lock().unwrap();
    if let Some(value) = db.get(&key.serialize()) {
        value.clone()
    } else {
        Value::Nil
    }
}
