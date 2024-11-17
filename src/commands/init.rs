use anyhow::{bail, Context, Result};
use chrono::{DateTime, Datelike, Local, Months, NaiveDate};
use clap::{builder::ValueParser, Arg, ArgAction, Command};
use regex::Regex;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub fn cli() -> Command {
    Command::new("init")
        .about("Initialize <file> to a new entry")
        .arg(
            Arg::new("file")
                .value_name("FILE")
                .value_parser(ValueParser::path_buf())
                .action(ArgAction::Set)
                .required(true)
                .help("File to be a new entry"),
        )
        .args([
            Arg::new("diary")
                .long("diary")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["note"])
                .help("Use a diary format"),
            Arg::new("note")
                .long("note")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["diary"])
                .help("Use a note format"),
        ])
}

pub fn exec(args: &clap::ArgMatches) -> Result<()> {
    let path = args
        .get_one::<PathBuf>("file")
        .with_context(|| format!("Failed to get {}", "file"))?;

    let opts = EntryOptions::new(
        path.to_path_buf(),
        args.get_flag("diary"),
        args.get_flag("note"),
    )?;

    let new_path = match opts.kind {
        EntryKind::Diary => diary_entry(&opts),
        EntryKind::Note => note_entry(&opts),
    }?;

    println!("{}", new_path.display());
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum EntryKind {
    Diary,
    Note,
}

#[derive(Debug)]
struct EntryOptions {
    path: PathBuf,
    cwd: PathBuf,
    kind: EntryKind,
}

impl EntryOptions {
    fn new(path: PathBuf, diary: bool, note: bool) -> Result<EntryOptions> {
        if !path.exists() {
            bail!("{} does not exist", path.display());
        }

        if !path.is_file() {
            bail!("{} is not a file", path.display());
        }

        if path.metadata()?.len() > 0 {
            bail!("{} is not empty", path.display());
        }

        let cwd = path.parent().unwrap_or(Path::new(".")).to_path_buf();

        let kind = match (diary, note) {
            (true, true) => bail!("can't specify both diary and note outputs"),
            (true, false) => EntryKind::Diary,
            (false, _) => EntryKind::Note,
        };

        Ok(EntryOptions { path, cwd, kind })
    }
}

fn diary_entry(opts: &EntryOptions) -> Result<PathBuf> {
    let pattern = Regex::new(r"^\d{6}\.md$")?;
    let latest = fs::read_dir(&opts.cwd)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = if entry.file_type().ok()?.is_file() {
                entry.file_name().into_string().ok()?
            } else {
                return None;
            };
            if pattern.is_match(&name) {
                let y = &name[..4];
                let m = &name[4..6];
                let s = format!("{}-{}-01", y, m);
                NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()
            } else {
                None
            }
        })
        .max();

    let today = Local::now().date_naive();
    let (y, m) = match latest {
        Some(date) => {
            if (date.year(), date.month()) < (today.year(), today.month()) {
                (today.year(), today.month())
            } else {
                let date = date + Months::new(1);
                (date.year(), date.month())
            }
        }
        None => (today.year(), today.month()),
    };

    let new_path = opts.cwd.join(format!("{:04}{:02}.md", y, m));
    fs::rename(&opts.path, &new_path)?;

    let file = File::create(&new_path)?;
    let metadata = file.metadata()?;
    let created: DateTime<Local> = metadata.created()?.into();
    let modified: DateTime<Local> = metadata.modified()?.into();

    let mut buf = BufWriter::new(file);
    buf.write_fmt(format_args!(
        "---
created_at: {}
updated_at: {}
---
",
        created.format("%Y-%m-%dT%H:%M:%S%:z"),
        modified.format("%Y-%m-%dT%H:%M:%S%:z"),
    ))?;

    let mut w = NaiveDate::from_ymd_opt(y, m, 1).unwrap().weekday();
    let n = ndays_in_month(y, m);
    for d in 1..=n {
        buf.write_fmt(format_args!(
            "
###### {:04}-{:02}-{:02}-{}

　
",
            y,
            m,
            d,
            w.to_string().to_lowercase(),
        ))?;
        w = w.succ();
    }

    buf.flush()?;
    Ok(new_path)
}

fn note_entry(opts: &EntryOptions) -> Result<PathBuf> {
    let created = opts.path.metadata()?.created()?;
    let id = ulid::Ulid::from_datetime(created);

    let new_path = opts.cwd.join(format!("{}.md", id));
    fs::rename(&opts.path, &new_path)?;

    let file = File::create(&new_path)?;
    let metadata = file.metadata()?;
    let created: DateTime<Local> = metadata.created()?.into();
    let modified: DateTime<Local> = metadata.modified()?.into();

    let mut buf = BufWriter::new(file);
    buf.write_fmt(format_args!(
        "---
created_at: {}
updated_at: {}
title:
tags:
---
",
        created.format("%Y-%m-%dT%H:%M:%S%:z"),
        modified.format("%Y-%m-%dT%H:%M:%S%:z"),
    ))?;

    buf.flush()?;
    Ok(new_path)
}

fn ndays_in_month(year: i32, month: u32) -> u32 {
    let (y, m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let date = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
    date.pred_opt().unwrap().day()
}
