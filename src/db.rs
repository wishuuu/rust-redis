use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use crate::resp::Value;

pub struct Entry {
    value: Value,
    expires_at: Option<SystemTime>,
}

impl Entry {
    pub fn new(value: Value, expires_at: Option<Duration>) -> Self {
        Entry {
            value,
            expires_at: if let Some(d) = expires_at {
                SystemTime::now().checked_add(d)
            } else {
                None
            },
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expire) = self.expires_at {
            if SystemTime::now() >= expire {
                return true
            }
        }
        false
    }
}

pub type Db = Arc<Mutex<HashMap<String, Entry>>>;

#[derive(Clone)]
pub struct DataLayer {
    db: Db,
}

impl DataLayer {
    pub fn new() -> Self {
        DataLayer {
            db: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_value(self, key: Value, value: Value, duration: Option<Duration>) -> Value {
        let mut db = self.db.lock().unwrap();
        let entry = Entry::new(value, duration);
        db.insert(key.clone().serialize(), entry);
        Value::SimpleString("OK".to_string())
    }

    pub fn get_value(self, key: Value) -> Value {
        let db = self.db.lock().unwrap();
        if let Some(value) = db.get(&key.serialize()) {
            if !value.is_expired() {
                return value.value.clone();
            }
        }
        Value::Nil
    }
}
