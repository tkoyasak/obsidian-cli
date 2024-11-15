use clap::{ArgMatches, Command};

mod new;

pub fn cli() -> Vec<Command> {
    vec![new::cli()]
}

type Exec = fn(&ArgMatches) -> anyhow::Result<()>;

pub fn infer(cmd: &str) -> Option<Exec> {
    match cmd {
        "new" => Some(new::exec),
        _ => None,
    }
}
