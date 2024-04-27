use std::{env, time::Duration};

use anyhow::Result;
use db::DataLayer;
use info::{Info, InfoLayer, Role};
use resp::{RespHandler, Value};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

mod db;
mod info;
mod resp;

#[tokio::main]
async fn main() {
    let args = env::args().into_iter();
    let info = Info::new().from_args(args);

    let info = InfoLayer::new(info);

    let listener = TcpListener::bind(format!("127.0.0.1:{}", info.info.lock().unwrap().port))
        .await
        .unwrap();

    let data_layer = DataLayer::new();

    let i = info.info.lock().unwrap();

    if let Role::Slave(master_socket) = i.replication.role {
        println!("Connecting to {:?} master", master_socket);
        let mut stream = TcpStream::connect(master_socket).await.unwrap();

        let _ = stream
            .write_all(
                Value::Array(Vec::from([Value::BulkString("ping".to_string())]))
                    .serialize()
                    .as_bytes(),
            )
            .await
            .unwrap();
        let mut response = vec![0; 1024];
        let _ = stream.read(&mut response).await.unwrap();

        let _ = stream
            .write_all(
                Value::Array(Vec::from([
                    Value::BulkString("REPLCONF".to_string()),
                    Value::BulkString("listening-port".to_string()),
                    Value::BulkString(i.port.to_string()),
                ]))
                .serialize()
                .as_bytes(),
            )
            .await
            .unwrap();
        let mut response = vec![0; 1024];
        let _ = stream.read(&mut response).await.unwrap();

        let _ = stream
            .write_all(
                Value::Array(Vec::from([
                    Value::BulkString("REPLCONF".to_string()),
                    Value::BulkString("capa".to_string()),
                    Value::BulkString("sync2".to_string()),
                ]))
                .serialize()
                .as_bytes(),
            )
            .await
            .unwrap();
        let mut response = vec![0; 1024];
        let _ = stream.read(&mut response).await.unwrap();
    }

    loop {
        let stream = listener.accept().await;
        let data_layer = data_layer.clone();
        let info = info.clone();
        match stream {
            Ok((stream, _)) => {
                println!("accepted new connection");
                tokio::spawn(async move {
                    handle_request(stream, data_layer, info).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn handle_request(stream: TcpStream, db: DataLayer, info: InfoLayer) {
    let mut handler = RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();

        println!("Got value: {:?}", value);

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            match command.to_ascii_uppercase().as_str() {
                "PING" => Value::SimpleString("PONG".to_string()),
                "ECHO" => args.first().unwrap().clone(),
                "SET" => db.clone().set_value(
                    args.first().unwrap().clone(),
                    args[1].clone(),
                    extract_duration_ms(args),
                ),
                "GET" => db.clone().get_value(args.first().unwrap().clone()),
                "INFO" => info.info.lock().unwrap().clone().serialize(&args[0]),
                "REPLCONF" => Value::SimpleString("OK".to_string()),
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

fn extract_duration_ms(args: Vec<Value>) -> Option<Duration> {
    for i in 1..args.len() {
        match &args[i] {
            Value::BulkString(c) => {
                if c.to_ascii_uppercase() == "PX" {
                    if let Value::BulkString(foo) = &args[i + 1] {
                        return Some(Duration::from_millis(foo.parse::<u64>().unwrap_or(0)));
                    }
                }
            }
            _ => {}
        };
    }
    None
}
