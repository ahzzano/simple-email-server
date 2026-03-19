use std::env;

use tokio::{self, net::TcpListener};
use tokio_postgres::NoTls;

mod database;
mod server;
mod session;

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = database::Database::new();
    let connected = db.connect(&database_url).await;

    connected.test();
    server::run("0.0.0.0:2525", String::from("localhost")).await;
}
