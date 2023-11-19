use tokio::io::AsyncReadExt;
use tokio::io::{self};

pub struct TerminalContext {}

impl TerminalContext {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&mut self) {
        let mut stdin = io::stdin();
        loop {
            let mut buffer = vec![0; 1024];
            let n = match stdin.read(&mut buffer).await {
                Err(_) | Ok(0) => break,
                Ok(n) => n,
            };
            buffer.truncate(n);

            println!("{}", String::from_utf8(buffer).unwrap());
            //tx.unbounded_send(Message::binary(buf)).unwrap();
        }
    }
}
