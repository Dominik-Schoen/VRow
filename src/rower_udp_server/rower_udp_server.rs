use std::error::Error;
use std::sync::Arc;
use tokio::net::UdpSocket;

pub struct Server {
    socket: Arc<UdpSocket>,
    client_ip: String
}

/// Creates a socket on the provided port and stores a correlated client IP.
/// 
/// # Example
/// ```
/// let server = setup_udp_server("8080", "127.0.0.1:8081").await.expect("Error creating the Socket");
/// ```
pub async fn setup_udp_server(server_port: String, client_ip: String) -> Result<Server, Box<dyn Error>> {
    let adr = format!("{}:{}", "0.0.0.0", server_port);
    let socket = match UdpSocket::bind(adr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Socket-Error: {}", e);
            return Err(Box::new(e));
        }
    };
    socket.set_broadcast(true)?;
    let socket_arc = Arc::new(socket);
    let socket_ret = socket_arc.clone();
    println!("Listening on: {}", socket_arc.local_addr()?);

    let server = Server {
        socket: socket_ret,
        client_ip: client_ip,
    };

    Ok(server)
}


impl Server {
    /// Sends the provided string to the client of the server.
    /// 
    /// # Example
    /// ```
    /// let _ = udp_server.send_string("Hello world");
    /// ```
    pub async fn send_string(&self, text:String) -> Result<(), Box<dyn Error>> {
        let Server {
            socket,
            client_ip,
        } = self;

        let sent = socket.send_to(text.as_bytes(), client_ip).await;
        match sent {
            Ok(_) => println!("Sent to {}: {}", client_ip, text),
            Err(e) => eprintln!("Sending-Error: {}", e),
        }
    
        Ok(())
    }
}
