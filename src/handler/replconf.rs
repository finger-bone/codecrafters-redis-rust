use std::sync::Arc;

use anyhow::{bail, Error};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio::sync::RwLock;

use crate::{protocol::RObject, Config};

pub async fn handle_replconf(
    args: &Vec<RObject>,
    stream: &mut TcpStream,
    _config: Arc<RwLock<Config>>
) -> Result<(), Error> {
    let target = match args.get(1) {
        Some(RObject::BulkString(s)) => s,
        _ => bail!("No configurable target found")
    };

    match target.as_str() {
        "listening-port" => {

        },
        "capa" => {

        }
        _ => bail!("Unrecognized replconf target")
    }

    
    stream.write(
        RObject::SimpleString("OK".to_string()).to_string().as_bytes()
    ).await.expect("Failed to respond to replconf");
    Ok(())
}