use std::sync::{Arc, Mutex};

use crate::trade::bank::Bank;



pub struct Exchange {
    pub bank: Arc<Mutex<Bank>>
}

impl Exchange {
    pub fn new() -> Self {
        Self { bank: Arc::new(Mutex::new(Bank::new())) }
    }
}