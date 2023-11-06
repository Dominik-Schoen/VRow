use std::process;
use std::env;
use btleplug::{platform::Adapter, platform::Peripheral};
use crate::rower_connector::connector::{connect_to_performance_monitor, get_ble_adapter_list};
use crate::rower_udp_server::rower_udp_server::setup_udp_server;

mod rower_connector;
mod rower_udp_server;
mod utils;

#[tokio::main]
async fn main() {
    // UDP Server
    let server_port = env::args()
        .nth(1)
        .unwrap_or_else(|| "8080".to_string());

    let client_ip = env::args()
        .nth(2)
        .unwrap_or_else(|| "255.255.255.255:8081".to_string());

    let udp_server = setup_udp_server(server_port, client_ip).await.unwrap();


    // Bluetooth
    // select adapter
    let adapter_list = get_ble_adapter_list().await.expect("Error getting Adapter");
    let adapter: Adapter = if adapter_list.len() > 1 {
         select_bluetooth_adapter(adapter_list).await
    } else {
        adapter_list.get(0).unwrap().1.clone()
    };
    
    // select PM5
    println!("Scanning... ");
    let peripheral_list = rower_connector::connector::scan_for_performance_monitors(adapter).await.expect("Error getting peripherals");
    let peripheral: Peripheral = if peripheral_list.len() > 1 {
        select_peripheral(peripheral_list).await
    } else {
        peripheral_list.get(0).unwrap().1.clone()
    };

    // connect to PM5
    for try_count in 0..4 {
        println!("Connecting to peripheral. Try {}", try_count);
        match connect_to_performance_monitor(peripheral.clone(), &udp_server).await {
            Ok(_) => break,
            Err(e) => {
                eprintln!("Error: {}", e);
                continue;
            },
        };
    }


    // Main loop
    println!("Ready. Type 'q' to exit.");
    loop {
        let input: String = utils::typed_read_line_blocking().await.unwrap();
        match input.as_ref() {
            "q" => process::exit(0),
            &_ => println!("Unkown command: {}", input),
        }
    }
}


async fn select_bluetooth_adapter(adapter_list: Vec<(String, Adapter)>) -> Adapter {
    println!("Select the index of the bluetooth adapter to use:");
    for (pos, adapter) in adapter_list.iter().enumerate() {
        println!(" {} - {}", pos, adapter.0);
    }

    loop {
        let index : usize = utils::typed_read_line_blocking().await.unwrap();

        match adapter_list.get(index) {
            Some(adapter_tuple) => return adapter_tuple.1.clone(),
            None => {
                println!("Index out of bounds! Input a valid index!");
                continue;
            },
        };
    }
}


async fn select_peripheral(peripheral_list: Vec<(String, Peripheral)>) -> Peripheral {
    println!("Select the index of the peripheral to use:");
    for (pos, peripheral) in peripheral_list.iter().enumerate() {
        println!(" {} - {}", pos, peripheral.0);
    }

    loop {
        let index : usize = utils::typed_read_line_blocking().await.unwrap();

        match peripheral_list.get(index) {
            Some(peripheral_tuple) => return peripheral_tuple.1.clone(),
            None => {
                println!("Index out of bounds! Input a valid index!");
                continue;
            },
        };
    }
}

