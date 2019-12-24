use clap::{crate_authors, crate_version, App, ArgMatches};

use super::modules::Connect;

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
        .subcommands(vec![Connect::generate_subcommand()])
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
