use anyhow::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::protocol::RObject;

pub async fn handle_ping(stream: &mut TcpStream) -> Result<(), Error> {
    stream.write(
        RObject::SimpleString("PONG".to_string()).to_string().as_bytes()
    ).await.expect(
        "error writing response to stream when responding to PING"
    );
    Ok(())
}