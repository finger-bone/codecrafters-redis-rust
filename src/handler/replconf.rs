use std::sync::Arc;

use anyhow::{bail, Error};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio::sync::RwLock;

use crate::{protocol::RObject, Config};

pub async fn handle_replconf(
    args: &Vec<RObject>,
    stream: &mut TcpStream,
    config: Arc<RwLock<Config>>
) -> Result<(), Error> {
    let target = match args.get(1) {
        Some(RObject::BulkString(s)) => s,
        _ => bail!("No configurable target found")
    };

    match target.as_str() {
        "listening-port" => {
            stream.write(
                RObject::SimpleString("OK".to_string()).to_string().as_bytes()
            ).await.expect("Failed to respond to replconf");
        },
        "GETACK" => {
            stream.write(
                RObject::Array(vec![
                    RObject::BulkString("REPLCONF".to_string()),
                    RObject::BulkString("ACK".to_string()),
                    RObject::BulkString(config.read().await.consumed.to_string())
                ]).to_string().as_bytes()
            ).await.expect("Failed to respond to replconf GETACK");
        },
        "capa" => {
            stream.write(
                RObject::SimpleString("OK".to_string()).to_string().as_bytes()
            ).await.expect("Failed to respond to replconf");
        }
        _ => bail!("Unrecognized replconf target")
    }

    Ok(())
}