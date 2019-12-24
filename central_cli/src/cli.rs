extern crate clap;
use clap::{crate_authors, crate_version, App, Arg, ArgMatches, SubCommand};

pub fn get_cli_matches<'a>() -> ArgMatches<'a> {
    let app = generate_app();

    app.get_matches()
}

#[allow(deprecated)]
fn generate_app<'a, 'b>() -> App<'a, 'b> {
    App::new("central")
        .author(crate_authors!("\n"))
        .version(crate_version!())
        .about("A Murillo Management System")
        .subcommand(generate_connect())
}

#[allow(deprecated)]
fn generate_connect<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("connect")
        .author(crate_authors!("\n"))
        .version(crate_version!())
        .about("Connect to central station to send ad-hoc messages")
        .arg(
            Arg::with_name("interactive")
                .short("i")
                .takes_value(false)
                .required(true)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_generate_app() {
        let app = generate_app();

        assert_eq!(app.get_name(), "central")
    }
}
