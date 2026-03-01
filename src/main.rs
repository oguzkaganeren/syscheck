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

    /// Detay modu: hatalı servislerin son 5 log mesajını göster
    #[arg(short, long)]
    verbose: bool,

    /// Log başlangıç zamanı (örn: '1 hour ago', '2024-01-15 10:00')
    #[arg(long)]
    since: Option<String>,

    /// Log bitiş zamanı (örn: '2024-01-15 12:00')
    #[arg(long)]
    until: Option<String>,

    /// Belirli bir servisi filtrele (örn: nginx.service)
    #[arg(value_name = "SERVICE")]
    service: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let raw = if args.stdin {
        read_stdin()?
    } else {
        run_journalctl(args.boots, args.since.as_deref(), args.until.as_deref(), args.service.as_deref())?
    };

    let entries = parse_journal_lines(&raw);

    // Servis filtresi varsa tüm logları sakla, yoksa son 5 satır yeterli
    let max_logs = if args.service.is_some() { 0 } else { 5 };
    let summaries = summarize(&entries, max_logs);

    let svc_filter = args.service.as_deref().map(|s| s.to_lowercase());
    let shown: Vec<&_> = if let Some(ref svc) = svc_filter {
        summaries
            .iter()
            .filter(|s| s.name.to_lowercase().contains(svc.as_str()))
            .collect()
    } else if args.top == 0 {
        summaries.iter().collect()
    } else {
        summaries[..args.top.min(summaries.len())].iter().collect()
    };

    // Servis filtresi varken otomatik verbose; Ok servisleri de göster
    let verbose = args.verbose || args.service.is_some();

    display::print_header();
    for s in shown {
        display::print_service_row(s);
        if verbose && (args.service.is_some() || s.issue_count() > 0) {
            display::print_service_detail(s);
        }
    }
    display::print_footer(&summaries);

    Ok(())
}

fn run_journalctl(
    boots: i32,
    since: Option<&str>,
    until: Option<&str>,
    service: Option<&str>,
) -> Result<String> {
    let mut cmd_args = vec!["-o".to_string(), "json".to_string(), "--no-pager".to_string()];

    if boots != -1 {
        cmd_args.push(format!("-b -{}", boots));
    }
    if let Some(s) = since {
        cmd_args.push("--since".to_string());
        cmd_args.push(s.to_string());
    }
    if let Some(u) = until {
        cmd_args.push("--until".to_string());
        cmd_args.push(u.to_string());
    }
    if let Some(svc) = service {
        cmd_args.push("-u".to_string());
        cmd_args.push(svc.to_string());
    }

    let output = Command::new("journalctl")
        .args(&cmd_args)
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
