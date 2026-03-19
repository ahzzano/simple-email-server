use std::sync::Arc;

use tokio::{net::TcpListener, sync::Mutex};

use crate::{
    database::{Connected, Database},
    session::Session,
};

pub async fn run(addr: &str, hostname: String, db: Arc<Mutex<Database<Connected>>>) {
    println!("Running Server");
    let listener: TcpListener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let hostname = hostname.clone();
                let db_acc = db.clone();
                tokio::spawn(async move {
                    Session::new(stream, hostname, db_acc).run().await;
                });
            }
            Err(e) => {
                eprintln!("Failed to connect to {addr}");
            }
        }
    }
}
