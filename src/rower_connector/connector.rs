use std::error::Error;
use std::time::Duration;
use tokio::time;

use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::{Manager, Adapter};

#[derive(Debug)]
struct BluetoothConnectorError {
    message: String,
}

impl  Error for BluetoothConnectorError { }

impl std::fmt::Display for BluetoothConnectorError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}


pub async fn get_ble_adapters() -> Result<Vec<Adapter>, Box<dyn Error>> {
    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;

    if adapter_list.is_empty() {
        return Err(Box::new(BluetoothConnectorError {
            message: "Couldn't find bluetooth adapter".to_string(),
        }));
    }

    Ok(adapter_list)
}