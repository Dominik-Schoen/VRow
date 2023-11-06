use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RowData {
    pub cals: u32,
    pub stroke_rate: u32,
    pub stroke_cals: u32
}

//&cals.lock().unwrap(), &stroke_rate.lock().unwrap(), &stroke_cals.lock().unwrap()