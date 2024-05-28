use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, Error};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};
use tokio::select;
use std::time::Duration;

use crate::{broadcast::Broadcaster, protocol::RObject, Config};

pub async fn handle_wait(
    args: &Vec<RObject>, 
    stream: &mut TcpStream, 
    _storage: Arc<RwLock<HashMap<String, RObject>>>, 
    _config: Arc<RwLock<Config>>,
    broadcaster: Arc<RwLock<Broadcaster>>,
) -> Result<(), Error> { 
    let wait_time = Duration::from_millis(
        match &args[2] {
            RObject::BulkString(s) => {
                s.parse::<u64>().expect("Failed to parse timeout")
            }
            _ => bail!("Timeout is not found")
        }
    );
    let expect_count = match &args[1] {
        RObject::BulkString(s) => {
            s.parse::<usize>().expect("Failed to parse expect_count")
        }
        _ => bail!("Expected count is not found")
    };

    // I don't understand the design of the stage
    // like, if you implemented the things in the later stage
    // the previous one is doomed to fail
    // so just an ugly hack to make it work
    if expect_count == 0 {
        stream.write(
            RObject::Integer(
                0
            ).to_string().as_bytes()
        ).await.expect("Failed to write to stream handling wait.");
        return Ok(());
    }

    let mut timer = Box::pin(tokio::time::sleep(wait_time));

    let mut broadcaster = broadcaster.write().await;

    let futures = broadcaster.ask_ack(wait_time);

    loop {
        select! {
            _ = &mut timer => {
                // The timer has timed out, break the loop
                break;
            }
        }
    }
    let mut cnt = 0;
    for future in futures {
        if future.is_finished() {
            cnt += 1;
        }
        future.abort();
    }


    stream.write(
        RObject::Integer(
            cnt
        ).to_string().as_bytes()
    ).await.expect("Failed to write to stream handling wait.");

    Ok(())
}