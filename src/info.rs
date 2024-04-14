use std::net::SocketAddrV4;

use std::env::Args;

use crate::resp::Value;

#[derive(Debug, Clone, Copy)]
pub enum Role {
    Master,
    Slave(SocketAddrV4),
}

impl Role {
    fn serialize(self) -> String {
        match self {
            Role::Master => "role:master".to_string(),
            Role::Slave(_) => "role:slave".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Info {
    pub role: Role,
    pub port: u16,
}

impl Info {
    pub fn new() -> Self {
        Info {
            role: Role::Master,
            port: 6379,
        }
    }
    pub fn from_args(mut self, mut args: Args) -> Self {
        args.next();
        loop {
            if let Some("--port") = args.next().as_deref() {
                self.port = args.next().unwrap().parse().expect("port expects u16")
            } else if let Some("--replicaof") = args.next_back().as_deref() {
                self.role = Role::Slave(SocketAddrV4::new(
                    args.next().unwrap().parse().expect("YUOA SUCK"),
                    args.next().unwrap().parse().expect("YOUA SUCK 2"),
                ))
            } else {
                break;
            }
        }
        self
    }

    pub fn serialize(self, info_part: &Value) -> Value {
        match info_part {
            Value::BulkString(c) => match c.as_str() {
                "replication" => {
                    Value::BulkString(format!("# Replication\r\n{}", self.role.serialize()))
                }
                _ => Value::Nil,
            },
            c => {
                println!(
                    "Received value {:?} as serialization info key, panicking",
                    c
                );
                Value::Nil
            }
        }
    }
}
