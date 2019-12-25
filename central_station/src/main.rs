use std::env::var;
use std::error::Error;
use std::str;

use bytes::BytesMut;

use tokio::prelude::*;
use tokio::net::UnixListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let home = var("HOME")?;

    // TODO: Create path if it doesn't exist yet
    let mut listener = UnixListener::bind(format!("{}/.central/tmp/.modules", home))
        .expect("Couldn't bind address");

    let mut connection_count: u32 = 0;
    loop {
        let (mut socket, _) = listener.accept().await?;
        let connection_id = connection_count;
        connection_count += 1;

        tokio::spawn(async move {
            loop {
                let mut read_buffer = BytesMut::with_capacity(1024); 

                // For now we just echo out whatever is written
                match socket.read_buf(&mut read_buffer).await {
                    Ok(n) if n == 0 => {
                        println!("{}: Connection Closed", connection_id);
                        return;
                    },
                    Err(e) => {
                        println!("Error reading data: {}", e);
                        return;
                    },
                    _ => {}
                };

                println!("{}: {}", connection_id, str::from_utf8(&read_buffer[..]).unwrap());

                //TODO: Close Connection on `SIGINT`
            }
        });
    }
}
