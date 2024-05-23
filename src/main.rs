pub mod protocol;
pub mod handler;

use std::collections::HashMap;
use std::sync::Arc;

use structopt::StructOpt;
use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use tokio::spawn;
use tokio::sync::RwLock;

use crate::handler::handle;
use crate::protocol::RObject;

#[derive(StructOpt)]
struct Cli {
    port: Option<u64>
}

#[tokio::main]
async fn main() {
    let args = Cli::from_args();
    let port = args.port.unwrap_or(6379);

    let storage = Arc::new(RwLock::new(HashMap::<String, RObject>::new()));
    
    let listener = TcpListener::bind(
        format!("127.0.0.1:{}", port)
    ).await.unwrap();
    
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
                    handle(&buf[..s], &mut stream, Arc::clone(&storage))
                        .await.expect("error handling request");
                }
            }
        });
    }
}