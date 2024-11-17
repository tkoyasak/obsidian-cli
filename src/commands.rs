use clap::{ArgMatches, Command};

mod init;

pub fn cli() -> Vec<Command> {
    vec![init::cli()]
}

type Exec = fn(&ArgMatches) -> anyhow::Result<()>;

pub fn infer(cmd: &str) -> Option<Exec> {
    match cmd {
        "init" => Some(init::exec),
        _ => None,
    }
}
