use std::{collections::HashMap, sync::Arc, convert::Infallible};

use crate::{Client, Clients, websocket_server::handlers};
use futures::{FutureExt, StreamExt};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::{ws::{Message, WebSocket}, Filter};

pub fn setup_websocket_server() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    print!("Configuring routes... ");
    let ws_route = warp::path("ws")
     .and(warp::ws())
     .and(with_clients(clients.clone()))
     .and_then(handlers::ws_handler);
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

pub async fn client_connection(ws: WebSocket, clients: Clients) {
    println!("establishing client connection to {:?}", ws);

    // create sender and receiver
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            println!("error sending websocket msg: {}", e);
        }
    }));

    // client registration
    let uuid = Uuid::new_v4().simple().to_string();

    let new_client = Client {
        client_id: uuid.clone(),
        sender: Some(client_sender),
    };

    clients.lock().await.insert(uuid.clone(), new_client);

    // message handling
    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("error resceiving message from id {}: {}", uuid.clone(), e);
                break;
            }
        };
        client_msg(&uuid, msg, &clients).await;
    }

    // disconnect client
    clients.lock().await.remove(&uuid);
    println!("{} disconnected", uuid);
}


async fn client_msg(client_id: &str, msg: Message, clients: &Clients) {
    println!("Received message from {}: {:?}", client_id, msg);

    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    let locked = clients.lock().await;
    for (_, client) in locked.iter() {
        let a = client.sender.as_ref().unwrap();
        let _ = a.send(Ok(Message::text(format!("{}", message))));
    }
    return;

    /*if message == "ping" || message == "ping\n" {
        let locked = clients.lock().await;
        match locked.get(client_id) {
            Some(v) => {
                if let Some(sender) = &v.sender {
                    println!("sending pong");
                    let _ = sender.send(Ok(Message::text("pong")));
                }
            }
            None => return,
        }
        return;
    };*/
}