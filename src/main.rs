pub mod protocol;
pub mod handler;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use tokio::spawn;

use crate::handler::handle;
use crate::protocol::RObject;

#[tokio::main]
async fn main() {
    let storage = Arc::new(RwLock::new(HashMap::<String, RObject>::new()));
    
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    
    const BUFFER_SIZE: usize = 4096;
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        let storage = Arc::clone(&storage);
        spawn(async move {
            loop {
                let mut buf = [0; BUFFER_SIZE];
                let s = stream.read(&mut buf)
                    .await.expect("error reading from stream");
                if s != 0 {
                    handle(&buf[..s], &mut stream, &storage)
                        .await.expect("error handling request");
                }
            }
        });
    }
}