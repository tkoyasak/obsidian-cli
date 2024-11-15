use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Datelike, Local, Months, NaiveDate};
use clap::{builder::ValueParser, Arg, ArgAction, ArgMatches, Command};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub fn cli() -> Command {
    Command::new("new")
        .about("Create a new entry in <path>")
        .arg(
            Arg::new("path")
                .value_name("PATH")
                .value_parser(ValueParser::path_buf())
                .action(ArgAction::Set)
                .default_value(".")
                .help("Path to create a new entry"),
        )
        .args([
            Arg::new("diary")
                .long("diary")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["ulid", "uuid"])
                .help("Use a diary format"),
            Arg::new("ulid")
                .long("ulid")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["diary", "uuid"])
                .help("Use a note fromat with ulid [default]"),
            Arg::new("uuid")
                .long("uuid")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["diary", "ulid"])
                .help("Use a note format with uuid"),
        ])
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let path = args
        .get_one::<PathBuf>("path")
        .with_context(|| format!("Failed to get {}", "path"))?;
    println!("path: {:?}", path);

    if !path.exists() || !path.is_dir() {
        fs::create_dir_all(path)?;
    };

    if args.get_flag("diary") {
        let today = Local::now().date_naive();
        let next = today + Months::new(1);
        diary_entry(path, today, Some(next))?;
    } else {
        note_entry(path, args.get_flag("uuid"))?;
    }

    Ok(())
}

fn diary_entry(path: &Path, date: NaiveDate, next: Option<NaiveDate>) -> Result<()> {
    let file_name = format!("{}.md", date.format("%Y%m"));
    let file_path = path.join(file_name);

    if file_path.exists() {
        match next {
            Some(next_date) => diary_entry(path, next_date, None),
            None => Err(anyhow!("The file already exists in {:?}", path)),
        }
    } else {
        let file = fs::File::create(file_path)?;

        let y = date.year();
        let m = date.month();

        let metadata = file.metadata()?;
        let created_at: DateTime<Local> = metadata.created()?.into();
        let updated_at: DateTime<Local> = metadata.modified()?.into();

        let mut buf = BufWriter::new(file);

        buf.write_fmt(format_args!(
            "---
date: {:04}-{:02}
created_at: {}
updated_at: {}
---
",
            y,
            m,
            created_at.format("%Y-%m-%dT%H:%M:%S%:z"),
            updated_at.format("%Y-%m-%dT%H:%M:%S%:z"),
        ))?;

        let mut w = NaiveDate::from_ymd_opt(y, m, 1).unwrap().weekday();
        let n = ndays_in_month(y, m);

        for d in 1..=n {
            buf.write_fmt(format_args!(
                "
#### {:04}-{:02}-{:02}-{}

　
",
                y,
                m,
                d,
                w.to_string().to_lowercase()
            ))?;
            w = w.succ();
        }

        buf.flush()?;
        Ok(())
    }
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

fn note_entry(path: &Path, is_uuid: bool) -> Result<()> {
    let id = if is_uuid {
        uuid::Uuid::now_v7().to_string()
    } else {
        ulid::Ulid::new().to_string()
    };

    let file_path = path.join(format!("{}.md", id));
    let file = fs::File::create(&file_path)?;

    let metadata = file.metadata()?;
    let created_at: DateTime<Local> = metadata.created()?.into();
    let updated_at: DateTime<Local> = metadata.modified()?.into();

    let mut buf = BufWriter::new(file);

    buf.write_fmt(format_args!(
        "---
id: {}
created_at: {}
updated_at: {}
title:
tags:
---

",
        id,
        created_at.format("%Y-%m-%dT%H:%M:%S%:z"),
        updated_at.format("%Y-%m-%dT%H:%M:%S%:z"),
    ))?;

    buf.flush()?;
    Ok(())
}
