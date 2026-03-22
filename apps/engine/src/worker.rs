use crate::engine::Engine;




pub struct Worker {
    pub queue : String,
    pub symbol : String,
    pub engine: Engine
}

impl Worker {
    pub fn new(queue : String, symbol: String, engine : Engine) -> Self {
        Self { 
            queue,
            symbol,
            engine
        }
    }

    pub fn run_worker() {
        
    }
}