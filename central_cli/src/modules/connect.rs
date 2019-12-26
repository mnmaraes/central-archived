use std::io;
use std::io::prelude::*;

use bytes::Bytes;

use clap::{crate_authors, crate_version, App, Arg, ArgMatches, SubCommand};

use tokio::net::UnixStream;
use tokio::prelude::*;

pub struct Connect {}

impl Connect {
    pub fn matches(arg_matches: ArgMatches) -> Option<ConnectRunner> {
        if let Some(sub_matches) = arg_matches.subcommand_matches("connect") {
            return Some(ConnectRunner::from(sub_matches));
        }

        None
    }

    #[allow(deprecated)]
    pub fn generate_subcommand<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("connect")
            .author(crate_authors!("\n"))
            .version(crate_version!())
            .about("Connect to central station to send ad-hoc messages")
            .arg(
                Arg::with_name("interactive")
                    .short("i")
                    .takes_value(false)
                    .help("Starts the connection in interactive mode"),
            )
            .arg(
                Arg::with_name("message")
                    .short("m")
                    .takes_value(true)
                    .required(true)
                    .conflicts_with("interactive")
                    .help("Sends a single message and closes the connection"),
            )
    }
}

pub enum ConnectRunner {
    Interactive,
    Message(String),
}

impl ConnectRunner {
    pub fn run(&self) -> io::Result<()> {
        match self {
            Self::Interactive => self.run_interactive(),
            Self::Message(_) => self.run_message(),
        }
    }
}

impl ConnectRunner {
    #[tokio::main]
    async fn run_message(&self) -> io::Result<()> {
        // 1. Connect to running instance of `central_station`
        //  - Get Socket Address
        let home = std::env::var("HOME").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let addr = format!("{}/.central/tmp/.modules", home);

        let mut client = UnixStream::connect(addr).await.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // 2. Send stored message!
        let message = self.message_value().expect("Invariant Violation: Running message passing mode when no message is present");
        let mut buffer = Bytes::from(message);

        let mut written = 0;
        while written < buffer.len() {
            match client.write_buf(&mut buffer).await {
                Ok(n) => {
                    written += n;
                },
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e))
            };
        }

        Ok(())
    }

    #[tokio::main]
    async fn run_interactive(&self) -> io::Result<()> {
        // 1. Connect to running instance of `central_station`
        //  - Get Socket Address
        let home = std::env::var("HOME").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let addr = format!("{}/.central/tmp/.modules", home);

        let mut client = UnixStream::connect(addr).await.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // 2. Send stored message!
        loop {
            print!("=> ");
            io::stdout().flush().expect("Error flushing stdout. What's up with that?");

            let mut message = String::new();
            io::stdin().read_line(&mut message)
                .expect("Error reading line from stdin");
            message = message.trim_end_matches('\n').to_string();

            let mut buffer = Bytes::from(message);

            let mut written = 0;
            while written < buffer.len() {
                match client.write_buf(&mut buffer).await {
                    Ok(n) => {
                        written += n;
                    },
                    Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e))
                };
            }
        }
    }

    fn message_value(&self) -> Option<String> {
        match self {
            Self::Message(m) => Some(m.clone()),
            _ => None,
        }
    }
}

impl<'a> From<&ArgMatches<'a>> for ConnectRunner {
    fn from(arg_matches: &ArgMatches<'a>) -> ConnectRunner {
        if arg_matches.is_present("interactive") {
            return ConnectRunner::Interactive;
        }

        ConnectRunner::Message(String::from(arg_matches.value_of("message").unwrap()))
    }
}
