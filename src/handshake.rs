use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};


use anyhow::Error;
use tokio::net::TcpStream;
use crate::BUFFER_SIZE;
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
    
    let mut ping_response_buffer = [0; BUFFER_SIZE];
    stream.read(&mut ping_response_buffer).await.expect("Failed to receive ping response when handshaking");
    // let (ping_res, _) = RObject::decode(std::str::from_utf8(&ping_response_buffer).expect(
    //     "Failed to decode ping response when handshaking."
    // ), 0).expect("Failed to parse the ping response when handshaking.");

    // 2. s->m replconf listening-port <>
    // replconf cap psync2

    stream.write_all(
        RObject::Array(
            vec![
                RObject::BulkString("REPLCONF".to_string()),
                RObject::BulkString("listening-port".to_string()),
                RObject::BulkString(
                    format!(
                        "{}", config.read().await.working_port
                    )
                )
            ]
        ).to_string().as_bytes()
    ).await.expect("Failed to config listening port");

    let mut replconf_listening_port_response = [0; BUFFER_SIZE];
    stream.read(&mut replconf_listening_port_response).await.expect("Failed to receive response ");

    stream.write_all(
        RObject::Array(
            vec![
                RObject::BulkString("REPLCONF".to_string()),
                RObject::BulkString("capa".to_string()),
                RObject::BulkString("psync2".to_string())
            ]
        ).to_string().as_bytes()
    ).await.expect("Failed to config listening port");

    let mut replconf_capa_response = [0; BUFFER_SIZE];
    stream.read(&mut replconf_capa_response).await.expect("Failed to receive capa responose when handshaking");

    // 3. m->s psync ? -1
    stream.write_all(
        RObject::Array(
            vec![
                RObject::BulkString("PSYNC".to_string()),
                RObject::BulkString("?".to_string()),
                RObject::BulkString("-1".to_string())
            ]
        ).to_string().as_bytes()
    ).await.expect("Failed to send psync");

    // read until meet \r\n
    let mut psync_response = Vec::new();
    loop {
        let mut byte = [0; 1];
        stream.read_exact(&mut byte).await.expect("Failed to read byte");
        // read until we reach the \r\n
        if byte[0] == b'\n' && psync_response.last() == Some(&b'\r') {
            psync_response.pop(); // Remove the '\r'
            break;
        }
        psync_response.push(byte[0]);
    }


    eprintln!("PSYNC response: {}", String::from_utf8_lossy(&psync_response));

    eprintln!("Ready to receive RDB file");
    // Read the length of the RDB file
    let mut len_buf = Vec::new();
    // the first byte is the '$'
    let mut dollar = [0; 1];
    eprintln!("Reading the dollar");
    stream.read_exact(&mut dollar).await.expect("Failed to read byte");
    eprintln!("Reading the length");
    loop {
        let mut byte = [0; 1];
        stream.read_exact(&mut byte).await.expect("Failed to read byte");
        // read until we reach the \r\n
        if byte[0] == b'\n' && len_buf.last() == Some(&b'\r') {
            len_buf.pop(); // Remove the '\r'
            break;
        }
        len_buf.push(byte[0]);
    }
    let len_str = std::str::from_utf8(&len_buf).expect("Failed to decode RDB length").trim();
    let len: usize = len_str.parse().expect("Failed to parse RDB length");
    eprintln!("RDB length: {}", len);
    let mut rdb_buf = vec![0; len];
    stream.read_exact(&mut rdb_buf).await.expect("Failed to read RDB file");
    
    Ok(Some(stream))
}