mod parsing;

use std::env;

use bytes::Bytes;

use failure::{Error, ResultExt};

use futures::stream::StreamExt;

use rmpv;

use tokio::net::{TcpListener, UnixListener, UnixStream};
use tokio::prelude::*;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use parsing::{encode_value, MsgPackParser};

// Server/Client
pub fn create_server() -> Result<UnboundedReceiver<rmpv::Value>, Error> {
    // 0. Set up
    let (tx, rx) = unbounded_channel();

    // 1. Set up Unix listener
    let mut unix_listener: UnixListener =
        open_uds_listener().context("Failed to open Unix Listener")?;

    tokio::spawn({
        let tx = tx.clone();

        async move {
            // TODO: Handle Errors in here
            loop {
                let (mut socket, _) = unix_listener.accept().await.unwrap();

                tokio::spawn({
                    let tx = tx.clone();

                    async move {
                        let (read_stream, _) = socket.split();

                        let mut parser = MsgPackParser::new(read_stream);

                        while let Some(value) = parser.next().await {
                            tx.send(value).ok();
                        }
                    }
                });
            }
        }
    });
    // 2. Set up Tcp listener
    //let tcp_listener = Self::open_tcp_listener()
    //.context("Failed to open Tcp Listener")?;

    // 3. Merge message streams
    Ok(rx)
}

pub async fn create_client() -> Result<UnboundedSender<rmpv::Value>, Error> {
    let addr = get_uds_path()?;

    let (tx, mut rx) = unbounded_channel::<rmpv::Value>();
    let mut stream = UnixStream::connect(addr).await?;

    tokio::spawn(async move {
        loop {
            let message = match rx.recv().await {
                Some(m) => m,
                None => {
                    break;
                }
            };

            let encoded = encode_value(&message);
            let mut buffer = Bytes::from(encoded);

            while buffer.len() > 0 {
                match stream.write_buf(&mut buffer).await {
                    Err(e) => {
                        println!("Sending error: {}", e);
                        println!(
                            "Trying to send message: {}",
                            String::from_utf8(buffer.to_vec()).unwrap()
                        );
                    }
                    _ => {}
                };
            }
        }
    });

    Ok(tx)
}

fn open_tcp_listener() -> Result<TcpListener, Error> {
    // 1. Handle cases where port is already in use
    // 2. Set up connection closing on close
    unimplemented!()
}

fn open_uds_listener() -> Result<UnixListener, Error> {
    // TODO
    let path = get_uds_path()?;
    // 1. Handle cases where file exists
    // 2. Set up connection closing on close
    // 3. Return Listener
    UnixListener::bind(path).map_err(Error::from)
}

fn get_uds_path() -> Result<String, Error> {
    let home = env::var("HOME").context("Couldn't retrieve HOME env var")?;
    Ok(format!("{}/.central/.sock", home))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
