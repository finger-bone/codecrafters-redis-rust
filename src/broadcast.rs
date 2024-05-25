use anyhow::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use futures::future::try_join_all;

pub struct Broadcaster {
    pub subscribers: Vec<TcpStream>
}

impl Broadcaster {
    pub fn subscribe(&mut self, target: TcpStream) {
        self.subscribers.push(target);
    }

    pub async fn broadcast(&mut self, message: &[u8]) -> Result<(), Error>{
        let mut futures = Vec::new();

        for subscriber in &mut self.subscribers {
            let future = subscriber.write_all(message);
            futures.push(future);
        }

        match try_join_all(futures).await {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Failed to broadcast: {:?}", e);
                Err(Error::from(e))
            }
        }
    }
}

