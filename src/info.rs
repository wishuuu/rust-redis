use std::net::Ipv4Addr;

use crate::resp::Value;

#[derive(Debug, Clone, Copy)]
pub enum Role {
    Master,
    Slave(Ipv4Addr),
}

impl Role {
    fn serialize(self) -> String {
        match self {
            Role::Master => "role:master".to_string(),
            Role::Slave(_) => "role:slave".to_string()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Info {
    pub role: Role,
}

impl Info {
    pub fn new(role: Role) -> Self {
        Info {
            role
        }
    }

    pub fn serialize(self, info_part: &str) -> Value {
        match info_part.to_ascii_lowercase().as_str() {
            "replication" => Value::BulkString(format!("# Replication\r\n{}", self.role.serialize() )),
            _ => panic!()
        }
    }
}
