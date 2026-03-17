use std::env;

use tokio_postgres::NoTls;
use tokio::{self, net::TcpListener};

mod session;
mod server;

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let (db_client, connection) = tokio_postgres::connect(&database_url, NoTls)
    .await
    .expect("Failed to connect to postgres");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let row = db_client.query_one("SELECT 1", &[])
        .await
        .expect("Failed to execute test query");
    println!("Connection successful!");

    server::run("0.0.0.0:2525",String::from("localhost"));
}
