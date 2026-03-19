use std::{collections::VecDeque, sync::Arc};

use postgres::{NoTls, Socket, tls::NoTlsStream};
use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};
use tokio_postgres::{Client, Connection};

use crate::session::Mail;

pub struct Disconnected;
pub struct Connected {
    sender: Sender<Mail>,
    client: Client,
}

pub struct Database<State = Disconnected> {
    state: State,
}

impl Database<Disconnected> {
    pub fn new() -> Self {
        Self {
            state: Disconnected,
        }
    }

    pub async fn connect(self, database_url: &str) -> (Database<Connected>, Receiver<Mail>) {
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

        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let db = Database {
            state: Connected {
                sender: tx,
                client: db_client,
            },
        };

        (db, rx)
    }
}

impl Database<Connected> {
    pub async fn start_processing_loop(
        db: Arc<Mutex<Database<Connected>>>,
        mut rx: Receiver<Mail>,
    ) {
        let this = Arc::clone(&db);
        tokio::spawn(async move {
            while let Some(item) = rx.recv().await {
                let mut dblock = this.lock().await;
                dblock.process_mail(item).await;
            }
        });
    }

    pub async fn process_mail(&mut self, mail: Mail) {
        println!("Processing mail...");
        let tx = self
            .state
            .client
            .transaction()
            .await
            .expect("Unable to make transaction");

        let result = async {
            tx.execute(
                "INSERT INTO mails (sender, receiver, body) VALUES ($1, $2, $3)",
                &[&mail.sender, &mail.reciever, &mail.body],
            )
            .await?;

            Ok::<(), tokio_postgres::Error>(())
        }
        .await;

        match result {
            Ok(_) => tx.commit().await.expect("Unable to commit transaction"),
            Err(e) => tx.rollback().await.expect("Unable to rollback transaction"),
        }
        println!("Finished Processing mail");
    }

    pub async fn init_tables(&mut self) {
        let tx = self
            .state
            .client
            .transaction()
            .await
            .expect("Unable to build transaction");

        tx.execute(
            "CREATE TABLE IF NOT EXISTS mails (
                id  SERIAL PRIMARY KEY,
                sender TEXT NOT NULL,
                receiver TEXT NOT NULL,
                body TEXT
            )",
            &[],
        )
        .await
        .unwrap();

        tx.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id      SERIAL PRIMARY KEY,
                address TEXT NOT NULL
            )",
            &[],
        )
        .await
        .unwrap();

        match tx.commit().await {
            Ok(_) => {
                println!("Tables created")
            }
            Err(e) => {
                eprintln!("Error: {e}")
            }
        }
    }

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
        self.state
            .sender
            .send(m)
            .await
            .expect("Unable to send Mail to DB");
        println!("Email for processing...")
    }
}
