use tokio::{io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader}, net::{TcpStream, tcp::WriteHalf}};


enum Commands {
    Helo,
    Ehlo,
    Rcpt,
    Mail,
    Data,
    Quit,
}

enum SessionState { 
    Created,
    Connected,
    MailFrom(String),
    RcptTo(String, String),
    Data(String, String),
}

pub struct Session {
    stream: TcpStream,
    hostname: String,
    state: SessionState
}

pub struct Mail {
    sender: String,
    reciever: String,
    body: String,
}

async fn send<W>(writer: &mut W, cmd:&str) 
where 
    W: AsyncWriteExt + Unpin,
{
    writer.write_all(format!("{}\r\n", cmd).as_bytes()).await.unwrap();
}

impl Session {
    pub fn new(stream: TcpStream, hostname: String) -> Self {
        Self {
            stream, hostname, state: SessionState::Created
        }
    }

    pub async fn run(mut self) { 
        let peer = self.stream.peer_addr().unwrap();

        println!("New Connection from {}", peer);

        let (read, mut write) = self.stream.split();
        let mut reader = BufReader::new(read);
        self.state = SessionState::Connected;
        let mut line = String::new();

        send(&mut write, 
            &format!("220 {} ESMTP Ready", self.hostname)
        ).await;

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    println!("Connection closed by {}", peer);
                    break;
                }
                Err(e) => {
                    println!("Read error from {}", peer);
                    break;
                }
                Ok(_) => {}
            }

            let trimmed = line.trim();
            println!("C: {}", trimmed);

            if line.starts_with("HELO") {
                let (cmd, host)= line.split_once(" ").expect("No host name");
                send(&mut write,
                    &format!("Hello {}, I am glad to meet you", host)
                ).await;
                continue;
            }

            if line.starts_with("MAIL FROM:") {
                let f = line.clone();
                let sender = f.chars()
                    .skip(10)
                    .filter(|c| *c == '<' || *c == '>')
                    .collect();

                self.state = SessionState::MailFrom(sender);
                send(&mut write, "250 OK").await;
                println!("Received Sender");
                continue;
            }

            if line.starts_with("RCPT TO:") {
                if let SessionState::MailFrom(sender) = &self.state  {
                    let f = line.clone();
                    let receiver = f.chars()
                        .skip(8)
                        .filter(|c| *c == '<' || *c == '>')
                        .collect();
                    self.state = SessionState::RcptTo(sender.clone(), receiver);
                    send(&mut write, "250 OK").await;
                    println!("Recieved recipient");
                    continue;
                }
            }

            if line.starts_with("DATA") {
                if let SessionState::RcptTo(sender, recipient) = &self.state {
                    self.state = SessionState::Data(sender.clone(), recipient.clone());
                    println!("{} is now sending data", peer);
                    send(&mut write, "354 End data with <CR><LF>.<CR><LF>").await;
                    continue;
                }
            }
            if line.ends_with("\r\n") {
                if let SessionState::Data(sender, recipient) = &self.state {
                    self.state = SessionState::Connected;
                    println!("{} finished sending mail", peer);
                    send(&mut write, "250 OK").await;
                    continue;
                }
            }
        }
    }
}