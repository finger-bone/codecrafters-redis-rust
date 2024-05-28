pub mod protocol;
pub mod handler;
pub mod state;
pub mod handshake;
pub mod broadcast;

use std::collections::HashMap;
use std::sync::Arc;

use broadcast::Broadcaster;
use state::ServerRole;
use handler::HandleResult;
use structopt::StructOpt;
use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use tokio::spawn;
use tokio::sync::RwLock;

use crate::handler::handle;
use crate::protocol::RObject;
use crate::handshake::handshake;

pub use crate::state::State;
pub use crate::state::BUFFER_SIZE;


#[derive(StructOpt)]
struct Cli {
    #[structopt(default_value = "6379", long)]
    port: u64,
    #[structopt(default_value = "", long)]
    replicaof: String, 
    #[structopt(default_value = "/dir", long)]
    dir: String,
    #[structopt(default_value = "dump.rdb", long)]
    dbfilename: String,
}


#[tokio::main]
async fn main() {
    let args = Cli::from_args();
    let port = args.port;

    let state_data = State {
        role: if args.replicaof.len() > 0 { ServerRole::Slave } else { ServerRole::Master },
        master_replid: "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string(),
        master_repl_offset: 0,
        replica_of: args.replicaof.clone().replace(" ", ":"),
        working_port: port,
        consumed: 0,
        dir: args.dir.clone(),
        dbfilename: args.dbfilename.clone(),
    };

    let state = Arc::new(RwLock::new(state_data));

    let storage = Arc::new(RwLock::new(HashMap::<String, RObject>::new()));

    let broadcaster = Arc::new(RwLock::new(Broadcaster{ subscribers: vec![], broadcasted: 0}));

    let master_stream = handshake(Arc::clone(&state)).await.expect(
        "Handshake failed"
    );
    
    if master_stream.is_some() {
        let mut master_stream = master_stream.unwrap();
        let storage: Arc<RwLock<HashMap<String, RObject>>> = Arc::clone(&storage);
        let state = Arc::clone(&state);
        let broadcaster = Arc::clone(&broadcaster);
        spawn(async move {
            loop {
                let mut buf = [0; BUFFER_SIZE];
                let s = master_stream.read(&mut buf)
                    .await.expect("error reading from stream");
                if s != 0 {
                    match handle(&buf[..s], master_stream, Arc::clone(&storage), Arc::clone(&state), Arc::clone(&broadcaster))
                    .await.expect("error handling request") {
                        HandleResult::Normal(s) => master_stream = s,
                        HandleResult::Subscribed => break,
                    }
                }
            }
        });
    }

    let listener = TcpListener::bind(
        format!("127.0.0.1:{}", port)
    ).await.unwrap();

    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        let storage: Arc<RwLock<HashMap<String, RObject>>> = Arc::clone(&storage);
        let state = Arc::clone(&state);
        let broadcaster = Arc::clone(&broadcaster);
        spawn(async move {
            loop {
                let mut buf = [0; BUFFER_SIZE];
                let s = stream.read(&mut buf)
                    .await.expect("error reading from stream");
                if s != 0 {
                    match handle(&buf[..s], stream, Arc::clone(&storage), Arc::clone(&state), Arc::clone(&broadcaster))
                        .await.expect("error handling request") {
                            HandleResult::Normal(s) => stream = s,
                            HandleResult::Subscribed => break,
                        }
                }
            }
        });
    }
}