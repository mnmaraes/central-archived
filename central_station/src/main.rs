use std::error::Error;

use tokio::prelude::*;
use tokio::net::UnixListener;

fn main() {
}

#[allow(dead_code)]
fn use_shared_mem() -> Result<(), Box<dyn Error>> {
    Ok(())
}

#[allow(dead_code)]
async fn use_sockets() -> Result<(), Box<dyn Error>> {
    let mut listener = UnixListener::bind("~/.central/tmp/.modules")
        .expect("Couldn't bind address");

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            // TODO: Send data over socket
        });
    }
}
