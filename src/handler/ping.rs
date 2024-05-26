use std::sync::Arc;

use anyhow::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};

use crate::{config::ServerRole, protocol::RObject, Config};

pub async fn handle_ping(stream: &mut TcpStream, config: Arc<RwLock<Config>>) -> Result<(), Error> {
    if config.read().await.role == ServerRole::Master {
        stream.write(
            RObject::SimpleString("PONG".to_string()).to_string().as_bytes()
        ).await.expect(
            "error writing response to stream when responding to PING"
        );
    }
    Ok(())
}