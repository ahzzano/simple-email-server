use std::collections::VecDeque;

use postgres::{NoTls, Socket, tls::NoTlsStream};
use tokio_postgres::{Client, Connection};

use crate::session::Mail;

pub struct Disconnected;
pub struct Connected {
    client: Client,
}

pub struct Database<State = Disconnected> {
    email_queue: VecDeque<Mail>,
    state: State,
}

impl Database<Disconnected> {
    pub fn new() -> Self {
        Self {
            email_queue: VecDeque::new(),
            state: Disconnected,
        }
    }

    pub async fn connect(self, database_url: &str) -> Database<Connected> {
        println!("Connecting to Database");
        let (db_client, connection) = tokio_postgres::connect(&database_url, NoTls)
            .await
            .expect("Failed to connect to postgres");
        println!("Connection established...");

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        Database {
            email_queue: self.email_queue,
            state: Connected { client: db_client },
        }
    }
}

impl Database<Connected> {
    pub async fn init_tables(&self) {}
    pub async fn test(&self) {
        let _row = self
            .state
            .client
            .query_one("SELECT 1", &[])
            .await
            .expect("Failed to execute test query");
        println!("Connection successful!");
    }

    pub async fn add_email_to_queue(&mut self, m: Mail) {
        self.email_queue.push_back(m);
        println!("Email for processing...")
    }
}

fn process_mail(mail: Mail) {
    println!("Saving mail...");
}
