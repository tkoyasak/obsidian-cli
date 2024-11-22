use clap::{ArgMatches, Command};

mod init;
mod new;

pub fn cli() -> Vec<Command> {
    vec![init::cli(), new::cli()]
}

type Exec = fn(&ArgMatches) -> anyhow::Result<()>;

pub fn infer(cmd: &str) -> Option<Exec> {
    match cmd {
        "init" => Some(init::exec),
        "new" => Some(new::exec),
        _ => None,
    }
}
