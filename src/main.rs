use std::{collections::HashMap, convert::Infallible, sync::Arc, io::stdin, process, str::FromStr, any::type_name};
use core::result::Result as CoreResult;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};
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


async fn typed_read_line_blocking<T: FromStr>() -> CoreResult<T, Box<dyn std::error::Error>> {
    println!("Expecting input of type {}:", type_name::<T>());
    let stdin = tokio::io::stdin();
    let mut reader = FramedRead::new(stdin, LinesCodec::new());

    loop {
        let line = reader.next().await.unwrap().expect("Something went wrong reading the buffer");
        let parsed_input = line.parse::<T>();

        match parsed_input {
            Ok(input) => return Ok(input),
            Err(_) => {
                println!("Expected type {1}. Couldn't parse '{0}' to {1}! Try different input.", line, type_name::<T>());
                continue;
            },
        }
    }
}


fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}