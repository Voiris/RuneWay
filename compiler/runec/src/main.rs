mod cli;
mod driver;
mod link;

use std::process::ExitCode;

use cli::{Cli, ParseOutcome};

fn main() -> ExitCode {
    let cli = match Cli::parse(std::env::args_os().skip(1)) {
        Ok(ParseOutcome::Run(cli)) => cli,
        Ok(ParseOutcome::Help) => {
            print!("{}", cli::HELP);
            return ExitCode::SUCCESS;
        }
        Ok(ParseOutcome::Version) => {
            println!("runec {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Err(error) => {
            eprintln!("error: {error}\n\n{}", cli::USAGE);
            return ExitCode::from(2);
        }
    };

    match driver::run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(driver::DriverError::Message(error)) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
        Err(driver::DriverError::Reported) => ExitCode::FAILURE,
    }
}
