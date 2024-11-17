use anyhow::bail;
use clap::{ArgMatches, Command};

mod commands;

fn main() {
    let matches = Command::new("cli")
        .subcommands(commands::cli())
        .get_matches();

    match cli(&matches) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    };
}

fn cli(matches: &ArgMatches) -> anyhow::Result<()> {
    let (cmd, subcommand_args) = match matches.subcommand() {
        Some((cmd, args)) => (cmd, args),
        _ => {
            bail!("No subcommand provided");
        }
    };

    if let Some(exec) = commands::infer(cmd) {
        exec(subcommand_args)
    } else {
        bail!("Unknown subcommand: {}", cmd)
    }
}
