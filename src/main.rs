use std::io::{BufRead, IsTerminal, Write};

use anyhow::Result;
use clap::{Parser, ValueEnum};
use filters::Filters;

use self::record::LogRecord;

mod filters;
mod record;

fn main() -> Result<()> {
    let args = Args::parse();
    let stdout = std::io::stdout();
    let stdin = std::io::stdin().lock();
    match args.color {
        Color::Always | Color::Auto if stdout.is_terminal() => colored::control::set_override(true),
        _ => colored::control::set_override(false),
    }

    let mut filters = Filters::builder();
    filters.with_strict(args.strict);
    for host in args.host {
        filters.with_host(&host)?;
    }

    process_lines(stdin, stdout, filters.build()?)
}

/// caddy-pretty-print is a simple tool for nicely viewing caddy JSON logs.
#[derive(Debug, Parser)]
#[command(version, about, max_term_width = 120)]
struct Args {
    /// When to use terminal colors.
    #[arg(long, default_value = "auto")]
    color: Color,

    /// Suppress all but legal log lines. By default lines that cannot be parsed are passed
    /// through.
    #[arg(long)]
    strict: bool,

    /// Filter the log lines by `host` header value. This flag can be repeated to search for
    /// multiples hosts or the glob syntax can be used to search hosts matching a given pattern.
    #[arg(long)]
    host: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum Color {
    #[default]
    Auto,
    Always,
    Never,
}

fn process_lines<I, O>(input: I, mut output: O, filters: Filters) -> Result<()>
where
    I: BufRead,
    O: Write,
{
    for line in input.lines() {
        let line = line.unwrap();
        match serde_json::from_str::<LogRecord>(&line) {
            Ok(record) => {
                if filters.matches(&record) {
                    writeln!(output, "{}", record.format())?;
                }
            }
            Err(_) => {
                if !filters.is_strict() {
                    writeln!(output, "{line}")?;
                }
            }
        }
    }
    Ok(())
}
