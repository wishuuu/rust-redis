use std::net::{Ipv4Addr, SocketAddrV4};

use std::env::Args;
use std::sync::{Arc, Mutex};

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

#[derive(Debug, Clone)]
pub struct Info {
    pub port: u16,
    pub replication: ReplicationInfo,
}

#[derive(Debug, Clone)]
pub struct ReplicationInfo {
    pub role: Role,
    pub master_replid: String,
    pub master_repl_offset: u32,
}

#[derive(Debug, Clone)]
pub struct InfoLayer {
    pub info: Arc<Mutex<Info>>,
}

impl InfoLayer {
    pub fn new(info: Info) -> Self {
        InfoLayer {
            info: Arc::new(Mutex::new(info))
        }
    }
}

impl Info {
    pub fn new() -> Self {
        Info {
            replication: ReplicationInfo {
                role: Role::Master,
                master_replid: "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string(),
                master_repl_offset: 0,
            },
            port: 6379,
        }
    }
    pub fn from_args(mut self, mut args: Args) -> Self {
        args.next();
        loop {
            if let Some(c) = args.next().as_deref() {
                match c {
                    "--port" => self.port = args.next().unwrap().parse().expect("port expects u16"),
                    "--replicaof" => {
                        self.replication.role = Role::Slave(SocketAddrV4::new(
                            args.next()
                                .unwrap()
                                .parse()
                                .unwrap_or(Ipv4Addr::new(127, 0, 0, 1)),
                            args.next().unwrap().parse().unwrap_or(6379),
                        ))
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }
        self
    }

    pub fn serialize(self, info_part: &Value) -> Value {
        match info_part {
            Value::BulkString(c) => match c.as_str() {
                "replication" => Value::BulkString(format!(
                    "# Replication\r\n{}\r\nmaster_replid:{}\r\nmaster_repl_offset:{}",
                    self.replication.role.serialize(),
                    self.replication.master_replid,
                    self.replication.master_repl_offset
                )),
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
