use serde::Serialize;


#[derive(Serialize)]
pub struct Status {
    pub active : bool,
    pub engine_tps : u64
}