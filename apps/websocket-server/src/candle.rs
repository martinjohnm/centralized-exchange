use serde::Serialize;



#[derive(Default, Clone, Serialize, Debug)]
pub struct Candle {
    pub open : f64,
    pub high : f64,
    pub low  : f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp : u64
}

impl Candle {

    // this is called when a new trade comes 

    pub fn update(&mut self, price: f64, qty: f64, timestamp : u64) {
        // the default case 
        if self.open == 0.0 { self.open = price; self.high = price; self.low = price; }

        // update the high and low if there is a lowest or highest value came
        if price > self.high { self.high = price; }
        if price < self.low { self.low = price; }

        // set closing price are the current value
        self.close = price;
        
        // increase volume by incresing the quantity
        // self.volume += qty;
        self.timestamp = timestamp;
    }
}
