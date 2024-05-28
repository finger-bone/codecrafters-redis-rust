use std::{sync::Arc, usize};

use anyhow::Error;
use futures::future::join_all;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, time::timeout, time::Duration};
use futures::lock::Mutex;

use crate::{protocol::RObject, BUFFER_SIZE};

pub struct Broadcaster {
    pub subscribers: Vec<Arc<Mutex<TcpStream>>>,
    pub broadcasted: usize,
}

impl Broadcaster {

    pub fn subscribe(&mut self, target: TcpStream) {
        self.subscribers.push(Arc::new(Mutex::new(target)));
    }

    pub async fn broadcast(&mut self, message: &[u8]) -> Result<(), Error>{
        let futures = self.subscribers.iter().map(|subscriber| {
            let subscriber = Arc::clone(subscriber);
            let message = message.to_vec();
            tokio::spawn(async move {
                let mut subscriber = subscriber.lock().await;
                subscriber.flush().await.expect("Failed to flush subscriber");
                subscriber.write_all(&message).await.expect(
                    "Failed to write to subscriber"
                );
            })
        }).collect::<Vec<_>>();

        self.broadcasted += message.len();

        join_all(futures).await;

        Ok(())
    }

    pub fn ask_ack(&mut self, wait_time: Duration) -> Vec<tokio::task::JoinHandle<Option<usize>>> {
        // sends replconf GETACK * to all subscribers
        let futures = self.subscribers.iter().map(|subscriber| {
            let subscriber = Arc::clone(subscriber);
            tokio::spawn(async move {
                let mut subscriber = subscriber.lock().await;

                // subscriber.write_all(
                //     RObject::Array(
                //         vec![
                //             RObject::BulkString("replconf".to_string()),
                //             RObject::BulkString("GETACK".to_string()),
                //             RObject::BulkString("*".to_string()),
                //         ]
                //     ).to_string().as_bytes()
                // ).await.expect("Failed to write to subscriber");

                let timer = timeout(wait_time, async {
                    subscriber.write_all(
                        RObject::Array(
                            vec![
                                RObject::BulkString("replconf".to_string()),
                                RObject::BulkString("GETACK".to_string()),
                                RObject::BulkString("*".to_string()),
                            ]
                        ).to_string().as_bytes()
                    ).await.expect("Failed to write to subscriber");
                });

                match timer.await {
                    Ok(_) => {},
                    Err(_) => {
                        eprintln!("Failed to write to subscriber");
                        return None;
                    }
                }

                let mut buffer = [0; BUFFER_SIZE];
                let s = subscriber.read(&mut buffer).await.unwrap_or(0);

                if s == 0 {
                    eprintln!("No content read from subscriber");
                    return None;
                }

                let (parsed, consumed) = RObject::decode(
                    &String::from_utf8_lossy(&buffer[..s]).to_string(),
                    0
                ).expect("Failed to parse response");

                if consumed != s {
                    eprintln!("Failed to consume all bytes");
                    eprintln!(
                        "Consumed: {}, Buffer: {}",
                        consumed,
                        s
                    );
                }

                
                if let RObject::Array(a) = parsed {
                    match {
                        a.get(2).expect("Failed to get second element")
                    } {
                        RObject::BulkString(s) => {
                            return Some(s.parse::<usize>().expect("Failed to parse integer"))
                        },
                        _ => {
                            eprintln!("Failed to parse integer");
                            return None
                        }
                    }
                } else {
                    eprintln!("Failed to parse integer");
                    return None
                }
            })
        }).collect::<Vec<_>>();
        
        futures
    }
}

