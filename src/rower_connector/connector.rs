use std::collections::BTreeSet;
use std::error::Error;
use std::time::Duration;
use futures::future::join_all;
use tokio::time;
use tokio_stream::StreamExt;
use uuid::Uuid;

use btleplug::api::{Central, Manager as _, ScanFilter, Peripheral, Characteristic, CharPropFlags, Descriptor};
use btleplug::platform::{Manager, Adapter, Peripheral as PlatformPeripheral};

const UUID: Uuid = Uuid::from_u128(0x43E5); // TODO: check for the real number
const SERVICE_UUID: Uuid = Uuid::from_u128(0xCE060030_43E5_11E4_916C_0800200C9A66);
const ROWING_STATUS_1_UUID: Uuid = Uuid::from_u128(0xCE060031_43E5_11E4_916C_0800200C9A66);
const ROWING_STATUS_2_UUID: Uuid = Uuid::from_u128(0xCE060032_43E5_11E4_916C_0800200C9A66);

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

/// Returns a list of all available adapters combined with their description
/// 
/// # Example
/// ```
/// let adapter_list = get_ble_adapter_list().await.expect("Error getting Adapter");
/// ```
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

/// Creates a tuple of the adapter's decription and adapter
async fn get_formated_adapter_info(adapter: Adapter) -> (String, Adapter) {
    return (format!("{:?}", (adapter.adapter_info().await)), adapter);
}

/// Returns a list of all available PM5s combined with their name, if available.
/// 
/// # Example
/// ```
/// let peripheral_list = rower_connector::connector::scan_for_devices(adapter).await.expect("Error getting peripherals");
/// ```
pub async fn scan_for_performance_monitors(adapter: Adapter) -> Result<Vec<(String, PlatformPeripheral)>, Box<dyn Error>> {
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

    let peripheral_list: Vec<(String, PlatformPeripheral)> = join_all(info_futures).await;
    Ok(peripheral_list.into_iter().filter(|(name, _)| name.starts_with("PM")).collect())
}

/// Creates a tuple of the peripheral's name and the peripheral
async fn get_peripheral_info(peripheral: PlatformPeripheral) -> (String, PlatformPeripheral) {
    return (format!("{:?}", peripheral.properties().await.unwrap().unwrap().local_name), peripheral);
}

/// Connects to the provided PM5.
pub async fn connect_to_performance_monitor(peripheral: PlatformPeripheral) -> Result<(), Box<dyn Error>> {
    peripheral.connect().await?;
    
    discover_PM_services(peripheral.clone()).await;
    
    // TODO: change based on scan
    let characteristic : Characteristic = Characteristic { 
        uuid: UUID, // TODO: change 
        service_uuid: SERVICE_UUID, 
        properties: CharPropFlags::READ, 
        descriptors: BTreeSet::from([
            Descriptor {
                uuid:  Uuid::from_u128(0x0031),
                service_uuid: SERVICE_UUID,
                characteristic_uuid: ROWING_STATUS_1_UUID,
            },
            Descriptor {
                uuid:  Uuid::from_u128(0x0032),
                service_uuid: SERVICE_UUID,
                characteristic_uuid: ROWING_STATUS_2_UUID,
            },
        ]) 
    };

    peripheral.subscribe(&characteristic).await?;
    let mut notification_stream = peripheral.notifications().await?;
    while let Some(data) = notification_stream.next().await {
        println!(
            "Received data from [{:?}]: {:?}",
            data.uuid, data.value
        );
    }    
    
    Ok(())
}

async fn discover_PM_services(peripheral: PlatformPeripheral) {
    peripheral.discover_services().await.expect("Error discovering");
    for characteristic in peripheral.characteristics() {
        println!("char {}", characteristic)
    }
}