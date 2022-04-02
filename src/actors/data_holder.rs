use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct DataHolder {
    pub req_count: Arc<Mutex<u32>>,
}

impl DataHolder {
    pub fn new() -> Self {
        Self {
            req_count: Arc::new(Mutex::new(0)),
        }
    }
}
