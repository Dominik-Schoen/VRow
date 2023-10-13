use std::error::Error;
use std::time::Duration;
use futures::future::join_all;
use tokio::time;

use btleplug::api::{Central, Manager as _, ScanFilter, Peripheral};
use btleplug::platform::{Manager, Adapter, Peripheral as PlatformPeripheral};

#[derive(Debug)]
struct BluetoothConnectorError {
    message: String,
}

impl Error for BluetoothConnectorError { }

impl std::fmt::Display for BluetoothConnectorError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

pub async fn get_ble_adapter_list() -> Result<Vec<(String, Adapter)>, Box<dyn Error>> {
    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;

    if adapter_list.is_empty() {
        return Err(Box::new(BluetoothConnectorError {
            message: "Couldn't find bluetooth adapter".to_string(),
        }));
    }

    let info_futures = adapter_list.into_iter()
        .map(|adapter| get_formated_adapter_info(adapter));
    let adapter_str_list = join_all(info_futures).await;
    Ok(adapter_str_list)
}

async fn get_formated_adapter_info(adapter: Adapter) -> (String, Adapter) {
    return (format!("{:?}", (adapter.adapter_info().await)), adapter);
}


pub async fn scan_for_devices(adapter: Adapter) -> Result<Vec<(String, PlatformPeripheral)>, Box<dyn Error>> {
    adapter.start_scan(ScanFilter::default()).await.expect("Can't scan");
    time::sleep(Duration::from_secs(10)).await; // TODO: make event driven
    let peripherals = adapter.peripherals().await?;

    if peripherals.is_empty() {
        return Err(Box::new(BluetoothConnectorError {
            message: "BLE peripheral devices were not found, sorry.".to_string(),
        }));
    }

    let info_futures = peripherals.into_iter()
        .map(|peripheral| get_peripheral_info(peripheral));

    let peripheral_list = join_all(info_futures).await;
    Ok(peripheral_list)
}

pub async fn connect_to_peripheral(peripheral: PlatformPeripheral) {
    peripheral.connect().await;
}

async fn get_peripheral_info(peripheral: PlatformPeripheral) -> (String, PlatformPeripheral) {
    return (format!("{:?}", peripheral.properties().await.unwrap().unwrap().local_name), peripheral);
}
