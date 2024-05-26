use anyhow::Error;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use futures::future::try_join_all;

use crate::{protocol::RObject, BUFFER_SIZE};

pub struct Broadcaster {
    pub subscribers: Vec<TcpStream>
}

impl Broadcaster {
    pub fn subscribe(&mut self, target: TcpStream) {
        self.subscribers.push(target);
    }

    pub async fn broadcast(&mut self, message: &[u8]) -> Result<(), Error>{
        println!("Scheduled to broadcast: {:?}", std::str::from_utf8(message).unwrap());
        
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

    pub async fn check_sync(&mut self, expect_bytes: usize) -> Result<usize, Error> {
        let mut count = 0;
        // for each subscriber, sends REPLCONF GETACK *
        // then check if the response is equal to the expected bytes
        for subscriber in &mut self.subscribers {
            subscriber.write_all(
                RObject::Array(
                    vec![
                        RObject::BulkString("REPLCONF".to_string()),
                        RObject::BulkString("GETACK".to_string()),
                        RObject::BulkString("*".to_string())
                    ]
                ).to_string().as_bytes()
            ).await.expect("Failed to send REPLCONF GETACK");

            let mut buffer = [0; BUFFER_SIZE];
            subscriber.read(&mut buffer).await.expect("Failed to read REPLCONF GETACK response");
            eprintln!("Received REPLCONF GETACK response: {:?}", String::from_utf8_lossy(buffer.as_ref()));
            let (response, _) = RObject::decode(std::str::from_utf8(&buffer).expect("Failed to decode REPLCONF GETACK response"), 0).expect("Failed to parse REPLCONF GETACK response");
            // response should be an integer
            // add one to count if that equals to the expected bytes
            if let RObject::Integer(i) = response {
                if i == expect_bytes as i64 {
                    count += 1;
                }
            }
        }
    
        Ok(count)
    }
}

