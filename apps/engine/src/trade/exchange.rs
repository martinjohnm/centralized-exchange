use std::{sync::{Arc, Mutex}, thread};

use crate::trade::{bank::{self, Bank}, engine::MatchingEngine};



pub struct Exchange {
    pub bank: Arc<Mutex<Bank>>
}

impl Exchange {
    
    pub fn new() -> Self {
        Self { bank: Arc::new(Mutex::new(Bank::new())) }
    }
}