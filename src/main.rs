mod display;
mod journal;
mod summary;

use std::io::{self, Read};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use clap::Parser;

use journal::parse_journal_lines;
use summary::summarize;

/// Linux sistem sağlık kontrol aracı.
/// journalctl loglarını parse ederek servisleri hata/uyarı sayısına göre özetler.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Gösterilecek maksimum servis sayısı (0 = tümü)
    #[arg(short = 'n', long, default_value_t = 20)]
    top: usize,

    /// Kaç önceki boot'u dahil et (-1 = tümü, 0 = mevcut boot)
    #[arg(short, long, default_value_t = 0)]
    boots: i32,

    /// Stdin'den oku (pipe kullanımı için)
    #[arg(long)]
    stdin: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let raw = if args.stdin {
        read_stdin()?
    } else {
        run_journalctl(args.boots)?
    };

    let entries = parse_journal_lines(&raw);
    let summaries = summarize(&entries);

    let shown: &[_] = if args.top == 0 {
        &summaries
    } else {
        &summaries[..args.top.min(summaries.len())]
    };

    display::print_header();
    for s in shown {
        display::print_service_row(s);
    }
    display::print_footer(&summaries);

    Ok(())
}

fn run_journalctl(boots: i32) -> Result<String> {
    let boot_arg = if boots == -1 {
        "--no-pager".to_string()
    } else {
        format!("-b -{}", boots)
    };

    let output = Command::new("journalctl")
        .args(["-o", "json", &boot_arg, "--no-pager"])
        .stdout(Stdio::piped())
        .output()
        .context("journalctl çalıştırılamadı")?;

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .context("stdin okunamadı")?;
    Ok(buf)
}
