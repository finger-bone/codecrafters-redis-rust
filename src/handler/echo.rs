use anyhow::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::protocol::{self, RObject};

pub async fn handle_echo(args: &Vec<RObject>, stream: &mut TcpStream) -> Result<(), Error> {
    if let protocol::RObject::BulkString(s) = args.get(1).ok_or_else(|| anyhow::anyhow!("Missing argument for ECHO"))? {
        stream.write(
            RObject::BulkString(s.clone()).to_string().as_bytes()
        ).await.expect(
            "error writing response to stream when responding to ECHO"
        );
    } else {
        anyhow::bail!("Expected bulk string as argument for ECHO");
    }
    Ok(())
}