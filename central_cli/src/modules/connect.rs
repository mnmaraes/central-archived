use clap::{crate_authors, crate_version, App, Arg, ArgMatches, SubCommand};

pub struct Connect {}

pub enum ConnectRunner {
    Interactive,
    Message(String),
}

impl ConnectRunner {
    pub fn run(&self) {
        match self {
            Self::Interactive => println!("TODO: Interactive Mode"),
            Self::Message(m) => println!("TODO: Message: {}", m),
        };
    }
}

impl<'a> From<&ArgMatches<'a>> for ConnectRunner {
    fn from(arg_matches: &ArgMatches<'a>) -> ConnectRunner {
        if arg_matches.is_present("interactive") {
            return ConnectRunner::Interactive;
        }

        return ConnectRunner::Message(String::from(arg_matches.value_of("message").unwrap()));
    }
}

impl Connect {
    pub fn matches<'a, 'b>(arg_matches: ArgMatches<'a>) -> Option<ConnectRunner> {
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
