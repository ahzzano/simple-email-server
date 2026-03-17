use tokio::net::TcpListener;

use crate::session::Session;


pub async fn run(addr: &str, hostname: String) {
    let listener: TcpListener = TcpListener::bind(addr).await
        .expect("Failed to bind to address");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let hostname = hostname.clone();
                tokio::spawn(async move {
                    Session::new(stream, hostname).run().await;
                });
            }
            Err(e) => {
                eprintln!("Failed to connect to {addr}");
            }
        }
    }
}