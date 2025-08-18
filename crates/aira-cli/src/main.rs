use std::{error::Error, process::ExitCode};

mod cmd;
mod utils;

fn main() -> ExitCode {
    let command = clap::command!()
        .subcommand_required(true)
        .subcommand(cmd::TiffDump)
        .max_term_width(100);

    let matches = command.get_matches();
    let subcommand = matches.subcommand().expect("Missing required subcommand");
    let result = match subcommand {
        (cmd::TiffDump::ID, matches) => cmd::TiffDump::run(matches),
        (cmd, _) => unreachable!("Unhandled command {cmd}"),
    };

    let Err(err) = result else {
        return ExitCode::SUCCESS;
    };

    let mut err: &dyn Error = err.as_ref();
    eprintln!("Error: {err}");
    while let Some(source) = err.source() {
        eprintln!("Caused by: {source}");
        err = source;
    }

    ExitCode::FAILURE
}
