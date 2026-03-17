use std::env;

use tokio_postgres::NoTls;
use tokio;

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let (client, connection) = tokio_postgres::connect(&database_url, NoTls)
    .await
    .expect("Failed to connect to postgres");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let row = client.query_one("SELECT 1", &[])
        .await
        .expect("Failed to execute test query");
    let result: i32 = row.get(0);
    println!("Connection successful!");
}
