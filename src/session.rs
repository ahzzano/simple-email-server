use tokio::{
    io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    net::{TcpStream, tcp::WriteHalf},
};

enum Commands {
    Helo(String),
    Ehlo,
    Rcpt(String),
    Mail(String),
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

// TODO: Rewrite to use PhantomData
pub struct Session {
    stream: TcpStream,
    hostname: String,
    state: SessionState,
    str_buffer: String,
}

pub struct Mail {
    sender: String,
    reciever: String,
    body: String,
}

async fn send<W>(writer: &mut W, cmd: &str)
where
    W: AsyncWriteExt + Unpin,
{
    writer
        .write_all(format!("{}\r\n", cmd).as_bytes())
        .await
        .unwrap();
}

fn parse_command(cmd: String) -> Option<Commands> {
    if cmd.starts_with("HELO") {
        let host = cmd.strip_prefix("HELO");
        return match host {
            Some(s) => Some(Commands::Helo(s.trim().to_string())),
            None => None,
        };
    }

    if cmd.starts_with("MAIL FROM") {
        let sender = cmd.strip_prefix("MAIL FROM");
        return match sender {
            Some(s) => Some(Commands::Mail(s.trim().to_string())),
            None => None,
        };
    }

    if cmd.starts_with("RCPT TO") {
        let recv = cmd.strip_prefix("RCPT TO");
        return match recv {
            Some(s) => Some(Commands::Rcpt(s.trim().to_string())),
            None => None,
        };
    }

    if cmd.starts_with("DATA") {
        return Some(Commands::Data);
    }

    None
}

impl Session {
    pub fn new(stream: TcpStream, hostname: String) -> Self {
        Self {
            stream,
            hostname,
            str_buffer: String::new(),
            state: SessionState::Created,
        }
    }

    pub async fn run(mut self) {
        let peer = self.stream.peer_addr().unwrap();

        println!("New Connection from {}", peer);

        let (read, mut write) = self.stream.split();
        let mut reader = BufReader::new(read);
        self.state = SessionState::Connected;
        let mut line = String::new();

        send(&mut write, &format!("220 {} ESMTP Ready", self.hostname)).await;

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

            let cmd = parse_command(line.clone());

            if let Some(c) = cmd {
                match c {
                    Commands::Helo(host) => {
                        send(
                            &mut write,
                            &format!("Hello {}, I am glad to meet you", host.trim()),
                        )
                        .await;
                    }
                    Commands::Mail(sender) => {
                        self.state = SessionState::MailFrom(sender);
                        send(&mut write, "250 OK").await;
                        println!("Received mail request")
                    }
                    Commands::Rcpt(recipient) => {
                        if let SessionState::MailFrom(sender) = &self.state {
                            self.state = SessionState::RcptTo(sender.clone(), recipient);
                            send(&mut write, "250 OK").await;
                            println!("Recieved recipient");
                        }
                    }
                    Commands::Data => {
                        if let SessionState::RcptTo(sender, recipient) = &self.state {
                            self.state = SessionState::Data(sender.clone(), recipient.clone());
                            println!("{} is now sending data", peer);
                            self.str_buffer.clear();
                            send(&mut write, "354 End data with <CR><LF>.<CR><LF>").await;
                            continue;
                        }
                    }
                    _ => {}
                }
            } else {
                if let SessionState::Data(_, _) = &self.state {
                    self.str_buffer.push_str(&line);
                }
            }

            if line.ends_with("\r\n") {
                if let SessionState::Data(sender, recipient) = self.state {
                    self.state = SessionState::Connected;
                    let m = Mail {
                        sender: sender.clone(),
                        reciever: recipient.clone(),
                        body: self.str_buffer.clone(),
                    };
                    println!("{} finished sending mail", peer);
                    send(&mut write, "250 OK").await;
                    continue;
                }
            }
        }
    }
}
