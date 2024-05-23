use std::sync::Arc;
use tokio::{io::AsyncWriteExt, sync::RwLock};

use anyhow::Error;
use tokio::net::TcpStream;

use crate::{protocol::RObject, Config};

pub async fn handshake(
    config: Arc<RwLock<Config>>
) -> Result<Option<TcpStream>, Error> {
    let address = config.read().await.replica_of.clone();
    if address.len() == 0 {
        return Ok(None);
    }
    
    let mut stream = TcpStream::connect(address).await.expect("Failed to connect to master");
    
    // 1. s->m ping
    stream.write_all(
        RObject::Array(
            vec![
                RObject::BulkString("PING".to_string())
            ]
        ).to_string().as_bytes()
    ).await.expect("Failed to ping when handshaking with master");
    

    Ok(Some(stream))
}