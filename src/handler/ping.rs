use std::sync::Arc;

use anyhow::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};

use crate::{state::ServerRole, protocol::RObject, State};

pub async fn handle_ping(stream: &mut TcpStream, state: Arc<RwLock<State>>) -> Result<(), Error> {
    if state.read().await.role == ServerRole::Master {
        stream.write(
            RObject::SimpleString("PONG".to_string()).to_string().as_bytes()
        ).await.expect(
            "error writing response to stream when responding to PING"
        );
    }
    Ok(())
}