use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use warp::{Filter, Rejection, ws::Message};

mod websocket_server;

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
    println!("Starting websocket server");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
    println!("Websocket server stopped!");
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}