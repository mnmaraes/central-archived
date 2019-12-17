use std::error::Error;
use std::str;

use bytes::BytesMut;

use tokio::prelude::*;
use tokio::net::UnixListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut listener = UnixListener::bind("/home/mnmaraes/.central/tmp/.modules")
        .expect("Couldn't bind address");
    let mut connection_count: u32 = 0;

    loop {
        let (mut socket, _) = listener.accept().await?;
        let connection_id = connection_count;
        connection_count = connection_count + 1;

        tokio::spawn(async move {
            loop {
                let mut read_buffer = BytesMut::with_capacity(1024); 

                println!("Reading from: {}", connection_id);

                // For now we just echo out whatever is written
                match socket.read_buf(&mut read_buffer).await {
                    Ok(n) if n == 0 => {
                        println!("Connection Closed");
                        return;
                    },
                    Err(e) => {
                        println!("Error reading data: {}", e);
                        return;
                    },
                    _ => println!("Data read successfully")
                };

                println!("Data: {}", str::from_utf8(&read_buffer[..]).unwrap());
            }
        });
    }
}
