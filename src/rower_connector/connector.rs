use std::collections::BTreeSet;
use std::error::Error;
use std::sync::Mutex;
use std::time::Duration;
use futures::future::join_all;
use tokio::time;
use tokio_stream::StreamExt;
use uuid::Uuid;

use btleplug::api::{Central, Manager as _, ScanFilter, Peripheral, Characteristic, CharPropFlags, Descriptor};
use btleplug::platform::{Manager, Adapter, Peripheral as PlatformPeripheral};

use crate::utils;

const SERVICE_UUID: Uuid = Uuid::from_u128(0xCE060030_43E5_11E4_916C_0800200C9A66);
//const ROWING_STATUS_1_UUID: Uuid = Uuid::from_u128(0xCE060031_43E5_11E4_916C_0800200C9A66);
const ROWING_STATUS_2_UUID: Uuid = Uuid::from_u128(0xCE060032_43E5_11E4_916C_0800200C9A66);
const ROWING_STATUS_3_UUID: Uuid = Uuid::from_u128(0xCE060033_43E5_11E4_916C_0800200C9A66);
const ROWING_STATUS_6_UUID: Uuid = Uuid::from_u128(0xCE060036_43E5_11E4_916C_0800200C9A66);

const DESCR_UUID: Uuid = Uuid::from_u128(0x00002902_0000_1000_8000_00805f9b34fb); // TODO: check with other rowers


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
    //Ok(peripheral_list)
    Ok(peripheral_list.into_iter().filter(|(name, _)| name.contains("PM")).collect())
}

/// Creates a tuple of the peripheral's name and the peripheral
async fn get_peripheral_info(peripheral: PlatformPeripheral) -> (String, PlatformPeripheral) {
    return (format!("{:?}", peripheral.properties().await.unwrap().unwrap().local_name), peripheral);
}

/// Connects to the provided PM5.
pub async fn connect_to_performance_monitor(peripheral: PlatformPeripheral) -> Result<(), Box<dyn Error>> {
    peripheral.connect().await?;
    
    // To enable scan of services, uncomment these lines
    //discover_pm_services(peripheral.clone()).await;
    //return Ok(());
    
    /*let characteristic_status_1 : Characteristic = Characteristic { 
        uuid: ROWING_STATUS_1_UUID,
        service_uuid: SERVICE_UUID, 
        properties: CharPropFlags::READ, 
        descriptors: BTreeSet::from([
            Descriptor {
                uuid:  DESCR_UUID,
                service_uuid: SERVICE_UUID,
                characteristic_uuid: ROWING_STATUS_1_UUID,
            }
        ]) 
    };*/

    let characteristic_status_2 : Characteristic = Characteristic { 
        uuid: ROWING_STATUS_2_UUID,
        service_uuid: SERVICE_UUID, 
        properties: CharPropFlags::READ, 
        descriptors: BTreeSet::from([
            Descriptor {
                uuid:  DESCR_UUID,
                service_uuid: SERVICE_UUID,
                characteristic_uuid: ROWING_STATUS_2_UUID,
            }
        ]) 
    };

    let characteristic_status_3 : Characteristic = Characteristic { 
        uuid: ROWING_STATUS_3_UUID,
        service_uuid: SERVICE_UUID, 
        properties: CharPropFlags::READ, 
        descriptors: BTreeSet::from([
            Descriptor {
                uuid:  DESCR_UUID,
                service_uuid: SERVICE_UUID,
                characteristic_uuid: ROWING_STATUS_3_UUID,
            }
        ]) 
    };

    let characteristic_status_6 : Characteristic = Characteristic { 
        uuid: ROWING_STATUS_6_UUID,
        service_uuid: SERVICE_UUID, 
        properties: CharPropFlags::READ, 
        descriptors: BTreeSet::from([
            Descriptor {
                uuid:  DESCR_UUID,
                service_uuid: SERVICE_UUID,
                characteristic_uuid: ROWING_STATUS_6_UUID,
            }
        ]) 
    };

    peripheral.subscribe(&characteristic_status_2).await?;
    peripheral.subscribe(&characteristic_status_3).await?;
    peripheral.subscribe(&characteristic_status_6).await?;
    let mut notification_stream = peripheral.notifications().await?;
    let cals = Mutex::new(0);
    let stroke_rate = Mutex::new(0);
    let stroke_cals = Mutex::new(0);
    
    while let Some(data) = notification_stream.next().await {
        let received_uuid = data.uuid;
        match received_uuid {
            ROWING_STATUS_2_UUID=>{
                let mut s = stroke_rate.lock().unwrap();
                *s = get_stroke_rate(&data.value);
                //println!("Stroke Rate {}", get_stroke_rate(&data.value));
            }
            ROWING_STATUS_3_UUID=>{
                let mut c = cals.lock().unwrap();
                *c = get_cals(&data.value);
                //println!("Cals {}", get_cals(&data.value));
            }
            ROWING_STATUS_6_UUID=>{
                let mut sr = stroke_cals.lock().unwrap();
                *sr = get_stroke_cals(&data.value);
                //println!("Stroke Cals {}", get_stroke_cals(&data.value));
            }
            _=> println!("Unkown uuid {}", received_uuid)
        }
        write_rowing_info_to_file(&cals.lock().unwrap(), &stroke_rate.lock().unwrap(), &stroke_cals.lock().unwrap());
        /*println!(
            "Received data from [{:?}]: {:?}",
            data.uuid, data.value
        );*/
    }    
    
    Ok(())
}

async fn discover_pm_services(peripheral: PlatformPeripheral) {
    peripheral.discover_services().await.expect("Error discovering");
    for characteristic in peripheral.characteristics() {
        println!("uuid {}", characteristic.uuid);
        println!("service uuid {}", characteristic.service_uuid);
        println!("properties {:#?}", characteristic.properties);
        println!("descri {:#?}", characteristic.descriptors);

        println!("char {}", characteristic);

        println!("-------------");
    }
    return;
}

fn write_rowing_info_to_file(cals: &u32, stroke_rate: &u32, stroke_cals: &u32) {
    utils::overwrite_file("rowerstatus.txt", format!("{} cal, {} strokes/m, {} cal/h", cals, stroke_rate, stroke_cals)).expect("couldn't write");
}

/*fn get_distance(data: &[u8]) -> f64 {
    let distance_high: u32 = data[5] as u32;
    let distance_mid: u32 = data[4] as u32;
    let distance_low: u32 = data[3] as u32;
    ((distance_low + distance_mid * 256 + distance_high * 65536) as f64) / 10.0
}*/

fn get_stroke_rate(data: &[u8]) -> u32 {
    let stroke_rate: u32 = data[5] as u32;
    return stroke_rate;
}

fn get_cals(data: &[u8]) -> u32 {
    let cals_high: u32 = data[7] as u32;
    let cals_low: u32 = data[6] as u32;
    return cals_high * 256 + cals_low
}

fn get_split_cals(data: &[u8]) -> u32 {
    let cals_high: u32 = data[13] as u32;
    let cals_low: u32 = data[12] as u32;
    return cals_high * 256 + cals_low
}

fn get_stroke_cals(data: &[u8]) -> u32 {
    let stroke_cals_high: u32 = data[7] as u32;
    let stroke_cals_low: u32 = data[6] as u32;
    return stroke_cals_high * 256 + stroke_cals_low
}