pub mod protocol;
pub mod handler;

use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use tokio::spawn;

use crate::handler::handle;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        spawn(async move {
            const BUFFER_SIZE: usize = 4096;
            let mut buf = [0; BUFFER_SIZE];
            let s = stream.read(&mut buf)
                .await.expect("error reading from stream");
            handle(&buf[..s], stream)
                .await.expect("error handling request");
        });
    }
}