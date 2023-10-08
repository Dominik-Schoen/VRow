use crate::{Clients, Result};
use crate::websocket_server::ws;
use warp::Reply;

pub async fn ws_handler(ws: warp::ws::Ws, clients: Clients) -> Result<impl Reply> {
    println!("ws_handler triggered");

    Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, clients)))
}