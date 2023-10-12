use std::{collections::HashMap, convert::Infallible, sync::Arc, io::stdin, process};
use btleplug::platform::Adapter;
use tokio::sync::{mpsc, Mutex};
use warp::{Filter, Rejection, ws::Message};

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
    
    // Bluetooth
    println!("Setting up bluetooth... ");
    let adapter_list = rower_connector::connector::get_ble_adapter_list().await.expect("Error getting Adapter");
    println!("Select the index of the bluetooth adapter to use:");
    for (pos, adapter) in adapter_list.iter().enumerate() {
        println!(" {} - {}", pos, adapter.0);
    }

    let adapter: Adapter = select_bluetooth_adapter(adapter_list);
    
    println!("Starting scanning... ");
    let peripheral_list = rower_connector::connector::scan_for_devices(adapter).await.expect("Error getting peripherals");

    // loop
    println!("Ready. Type 'q' to exit.");
    let input: String = utils::typed_read_line_blocking().await.unwrap();
    println!("{}", input);

    /*loop {
        let mut user_input = String::new();
        match stdin().read_line(&mut user_input) {
            Ok(input) => input,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };
        let user_input = user_input.trim_end_matches(&['\r', '\n'][..]);
        
        match user_input.as_ref() {
            "q" => process::exit(0),
            &_ => println!("Unkown command: {}", user_input),
        }
    }*/
}

fn select_bluetooth_adapter(adapter_list: Vec<(String, Adapter)>) -> Adapter {
    loop {
        
        let mut user_input = String::new();
        match stdin().read_line(&mut user_input) {
            Ok(input) => input,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };
        let user_input = user_input.trim_end_matches(&['\r', '\n'][..]);

        let adapter_option = adapter_list.iter().find(|(index, _)| index == user_input);
        match adapter_option {
            Some((_0, _1)) => return _1.clone(),
            None => {
                println!("Invalid input. Index out of bounds?");
                continue;
            },
        }
    }
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}