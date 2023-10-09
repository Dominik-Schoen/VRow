use std::{collections::HashMap, convert::Infallible, sync::Arc, io::stdin, process};
use tokio::sync::{mpsc, Mutex};
use warp::{Filter, Rejection, ws::Message};

mod websocket_server;
mod rower_connector;

#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

type Clients = Arc<Mutex<HashMap<String, Client>>>;
type Result<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {
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
    print!("Setting up Bluetooth. I need your help here!");
    rower_connector::connector::get_ble_adapters();

    println!("Ready. Type 'q' to exit.");
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
        
        match user_input.as_ref() {
            "q" => process::exit(0),
            &_ => println!("Unkown command: {}", user_input),
        }
    }
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}