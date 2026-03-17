use tokio::{io::{AsyncWrite, AsyncWriteExt, BufReader}, net::{TcpStream, tcp::WriteHalf}};

pub struct Session {
    stream: TcpStream,
    hostname: String,
}

enum SessionState { 
    Connected
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
            stream, hostname
        }
    }

    pub async fn run(mut self) { 
        let peer = self.stream.peer_addr().unwrap();

        println!("New Connection from {}", peer);

        let (read, mut write) = self.stream.split();
        let mut reader = BufReader::new(read);
        let mut state = SessionState::Connected;
        let mut line = String::new();

        send(&mut write, 
            &format!("220 {} ESMTP Ready", self.hostname)
        ).await
    }
}