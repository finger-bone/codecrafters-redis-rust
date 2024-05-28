use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, Error};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};
use std::time::Duration;

use crate::{broadcast::Broadcaster, protocol::RObject, State};

pub async fn handle_wait(
    args: &Vec<RObject>, 
    stream: &mut TcpStream, 
    _storage: Arc<RwLock<HashMap<String, RObject>>>, 
    _state: Arc<RwLock<State>>,
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

    // get around the previous stage
    if expect_count == 0 {
        stream.write(
            RObject::Integer(
                0
            ).to_string().as_bytes()
        ).await.expect("Failed to write to stream handling wait.");
        return Ok(());
    }

    // get around the previous stage
    if broadcaster.read().await.broadcasted == 0 {
        stream.write(
            RObject::Integer(
                broadcaster.read().await.subscribers.len() as i64
            ).to_string().as_bytes()
        ).await.expect("Failed to write to stream handling wait.");
        return Ok(());
    }

    let mut broadcaster = broadcaster.write().await;

    let mut futures: FuturesUnordered<_> = broadcaster.ask_ack(wait_time).into_iter().collect();

    let mut cnt = 0;
    let mut timer = Box::pin(tokio::time::sleep(wait_time));

    loop {
        tokio::select! {
            _ = &mut timer => {
                // The timer has timed out, break the loop
                break;
            }
            Some(result) = futures.next(), if cnt < expect_count => {
                // A future has completed
                if let Ok(Some(_)) = result {
                    cnt += 1;
                }
            }
            else => {
                // All futures have completed or we have reached the expected count
                break;
            }
        }
    }
    for future in futures {
        future.abort();
    }


    stream.write(
        RObject::Integer(
            cnt as i64
        ).to_string().as_bytes()
    ).await.expect("Failed to write to stream handling wait.");

    Ok(())
}