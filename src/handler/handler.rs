use anyhow::{bail, Error};
use tokio::net::TcpStream;

use crate::{handler::{handle_echo, handle_ping}, protocol};

pub async fn handle(request: &[u8], stream: &mut TcpStream) -> Result<(), Error> {
    
    let str_req = std::str::from_utf8(request)?;

    // eprintln!(
    //     "Handling request: {}",
    //     if true { str_req } else { "" }
    // );

    let (parsed, _) = protocol::RObject::decode(str_req, 0)?;

    eprintln!("Parsed request: {:#?}", parsed); 

    if let protocol::RObject::Array(a) = parsed {
        let command = match a.get(0)
            .ok_or_else(|| anyhow::anyhow!("Empty array"))? {
                protocol::RObject::SimpleString(s) => s,
                protocol::RObject::BulkString(s) => s,
                _ => bail!("Expected string as command"),
            };
        if command == "PING" {
            handle_ping(stream).await?;
        } else if command == "ECHO" {
            handle_echo(&a, stream).await?;
        } else {
            bail!("Unknown command: {}", command);
        }
    } else {
        bail!("Expected array as request");
    }

    Ok(())
}