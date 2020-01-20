mod codec;
mod parsing;
pub mod runtime;

pub use cliff_derive::*;

use std::{
    env, fs,
    io::ErrorKind,
    sync::{Arc, Mutex},
};

use bytes::Bytes;

use failure::{Error, ResultExt};

use futures::stream::StreamExt;

use rmpv;

use tokio::net::{
    unix::{ReadHalf, WriteHalf},
    UnixListener, UnixStream,
};
use tokio::prelude::*;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use tokio_util::codec::Framed;

use parsing::{encode_value, MsgPackParser};

pub use runtime::{Handler, Message};
use runtime::{Runtime, SelfStarter};

pub struct UnixConnection(Option<UnixStream>);
impl Message for UnixConnection {
    fn message_type(&self) -> String {
        "UnixConnection".to_string()
    }
}

impl UnixConnection {
    pub fn take_socket(&mut self) -> Option<UnixStream> {
        self.0.take()
    }
}

pub trait UnixServer: Sized {
    fn serve() -> Runtime<Self>;
}

impl<T: Handler<UnixConnection> + Default + Send + 'static> UnixServer for T {
    fn serve() -> Runtime<T> {
        let runtime = T::start();

        listen(&runtime);

        runtime
    }
}

fn listen<T: Handler<UnixConnection> + Default + Send + 'static>(runtime: &Runtime<T>) {
    let mut listener = open_uds_listener()
        .context("Failed to open Unix Listener")
        .unwrap();
    let cloned = runtime.clone();

    tokio::spawn(async move {
        let new_conn_stream = listener
            .incoming()
            .filter_map(|r: Result<_, _>| async { r.ok() })
            .then(|socket| forward_parsed(&cloned, socket));

        let mut pinned = Box::pin(new_conn_stream);
        while let Some(m) = pinned.next().await {
            cloned.send(m);
        }
    });
}

async fn forward_parsed<T: Handler<UnixConnection> + Default + Send + 'static>(
    runtime: &Runtime<T>,
    mut socket: UnixStream,
) -> UnixConnection {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        // TODO: Handle the Write side of this equation.
        // Need to figure out how to work with stream and subject
        // separately and the borrow/lifetime implications
        // NOTE: Trying to split these into different threads tends not to work out
        // It'd be better to have a wrapping struct that owns the socket
        // and manages the task splitting
        // Seems like an answer: https://docs.rs/tokio-util/0.2.0/tokio_util/codec/struct.Framed.html
        // As seen in: https://github.com/tokio-rs/tokio/issues/1840
        // Also: https://docs.rs/futures-preview/0.3.0-alpha.19/futures/macro.select.html
        let framed = Framed::new(socket, codec::MsgPackCodec {});

        let (subject, mut stream) = framed.split();
        let forwarded = rx.forward(subject);

        tokio::spawn(forwarded);
        // TODO: Probably not the parser we actually want
        // Ideally we'd be working with a message parser
        if let Some(Ok(next)) = stream.next().await {
            // TODO: Parse Message
        };
    });

    // TODO: Return tx here
    UnixConnection(None)
}

// Server/Client
pub fn create_server() -> Result<UnboundedReceiver<rmpv::Value>, Error> {
    // TODO: Set up connection closing on close
    // 0. Set up
    let (tx, rx) = unbounded_channel();

    // 1. Set up Unix listener
    let mut unix_listener = open_uds_listener().context("Failed to open Unix Listener")?;

    tokio::spawn({
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

    // 2. Return message streams
    Ok(rx)
}

fn open_uds_listener() -> Result<UnixListener, Error> {
    let path = get_uds_path()?;
    match UnixListener::bind(&path) {
        Ok(l) => Ok(l),
        Err(e) if e.kind() == ErrorKind::AddrInUse => {
            // 1. Handle cases where file exists
            // TODO: Handle it more gracefully (Ask user whether to force or abort)
            println!("A connection file already exists. Removing it.");
            fs::remove_file(&path)?;

            UnixListener::bind(&path).map_err(Error::from)
        }
        Err(e) => Err(Error::from(e)),
    }
}

pub async fn create_client() -> Result<UnboundedSender<rmpv::Value>, Error> {
    let addr = get_uds_path()?;

    let (tx, mut rx) = unbounded_channel::<rmpv::Value>();
    let mut stream = UnixStream::connect(addr).await?;

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            let encoded = encode_value(&message);
            let mut buffer = Bytes::from(encoded);

            while !buffer.is_empty() {
                if let Err(e) = stream.write_buf(&mut buffer).await {
                    println!("Sending error: {}", e);
                    println!(
                        "Trying to send message: {}",
                        String::from_utf8(buffer.to_vec()).unwrap()
                    );
                }
            }
        }
    });

    Ok(tx)
}

fn get_uds_path() -> Result<String, Error> {
    let home = env::var("HOME").context("Couldn't retrieve HOME env var")?;
    Ok(format!("{}/.central/.sock", home))
}
