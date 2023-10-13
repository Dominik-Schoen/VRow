use std::{collections::HashMap, convert::Infallible, sync::Arc, process};
use btleplug::{platform::Adapter, platform::Peripheral};
use tokio::sync::{mpsc, Mutex};
use warp::{Filter, Rejection, ws::Message};

use crate::rower_connector::connector::{connect_to_peripheral, get_ble_adapter_list};

mod websocket_server;
mod rower_connector;
mod utils;

#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

type Clients = Arc<Mutex<HashMap<String, Client>>>;
type Result<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {
    // websocket server
    setup_websocket_server();
    
    // Bluetooth
    let adapter_list = get_ble_adapter_list().await.expect("Error getting Adapter");
    println!("Select the index of the bluetooth adapter to use:");
    for (pos, adapter) in adapter_list.iter().enumerate() {
        println!(" {} - {}", pos, adapter.0);
    }
    let adapter: Adapter = select_bluetooth_adapter(adapter_list).await;
    
    println!("Scanning... ");
    let peripheral_list = rower_connector::connector::scan_for_devices(adapter).await.expect("Error getting peripherals");
    println!("Select the index of the peripheral to use:");
    for (pos, peripheral) in peripheral_list.iter().enumerate() {
        println!(" {} - {}", pos, peripheral.0);
    }
    let peripheral: Peripheral = select_peripheral(peripheral_list).await;
    for try_count in 0..4 {
        println!("Connecting to peripheral. Try {}", try_count);
        match connect_to_peripheral(peripheral.clone()).await {
            Ok(_) => break,
            Err(e) => {
                eprintln!("Error: {}", e);
                continue;
            },
        };
    }

    // loop
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

fn setup_websocket_server() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    print!("Configuring routes... ");
    let ws_route = warp::path("ws")
     .and(warp::ws())
     .and(with_clients(clients.clone()))
     .and_then(websocket_server::handlers::ws_handler);

    let routes = ws_route
     .with(warp::cors().allow_any_origin());
    println!("done");

    print!("Starting websocket server... ");
    tokio::spawn(async move{
        warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
    });
    println!("done");
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}