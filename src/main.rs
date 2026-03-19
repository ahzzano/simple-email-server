use std::{env, sync::Arc};

use tokio::{self, net::TcpListener, sync::Mutex};

use crate::database::Database;

mod database;
mod server;
mod session;

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = database::Database::new();
    let (mut db, rx) = db.connect(&database_url).await;
    db.test().await;
    db.init_tables().await;
    let to_share_db = Arc::new(Mutex::new(db));

    Database::start_processing_loop(to_share_db.clone(), rx).await;

    println!("Starting server...");
    server::run(
        "0.0.0.0:2525",
        String::from("localhost"),
        to_share_db.clone(),
    )
    .await;
}
