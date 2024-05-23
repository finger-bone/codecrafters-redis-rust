pub mod protocol;
pub mod handler;
pub mod config;

use std::collections::HashMap;
use std::sync::Arc;

use structopt::StructOpt;
use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use tokio::spawn;
use tokio::sync::RwLock;

use crate::handler::handle;
use crate::protocol::RObject;
pub use crate::config::Config;


#[derive(StructOpt)]
struct Cli {
    #[structopt(default_value = "6379", long)]
    port: u64,
    #[structopt(default_value = "", long)]
    replicaof: String, 
}


#[tokio::main]
async fn main() {
    let args = Cli::from_args();
    let port = args.port;

    let config_data = Config {
        role: if args.replicaof.len() > 0 { "slave".to_string() } else { "master".to_string() },
        master_replid: "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string(),
        master_repl_offset: 0,
    };

    let config = Arc::new(RwLock::new(config_data));

    let storage = Arc::new(RwLock::new(HashMap::<String, RObject>::new()));

    let listener = TcpListener::bind(
        format!("127.0.0.1:{}", port)
    ).await.unwrap();
    
    const BUFFER_SIZE: usize = 4096;
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        let storage = Arc::clone(&storage);
        let config = Arc::clone(&config);
        spawn(async move {
            loop {
                let mut buf = [0; BUFFER_SIZE];
                let s = stream.read(&mut buf)
                    .await.expect("error reading from stream");
                if s != 0 {
                    handle(&buf[..s], &mut stream, Arc::clone(&storage), Arc::clone(&config))
                        .await.expect("error handling request");
                }
            }
        });
    }
}