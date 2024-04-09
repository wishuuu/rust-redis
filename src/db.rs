use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::resp::Value;

pub type Db = Arc<Mutex<HashMap<String, Value>>>;


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

    pub fn set_value(self, key: Value, value: Value) -> Value {
        let mut db = self.db.lock().unwrap();
        db.insert(key.clone().serialize(), value.clone());
        Value::SimpleString("OK".to_string())
    }

    pub fn get_value(self, key: Value) -> Value {
        let db = self.db.lock().unwrap();
        if let Some(value) = db.get(&key.serialize()) {
            value.clone()
        } else {
            Value::Nil
        }
    }
}
