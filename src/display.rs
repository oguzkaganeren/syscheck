use colored::Colorize;

use crate::summary::{HealthStatus, ServiceSummary};

const COL_SERVICE: usize = 36;
const COL_COUNT: usize = 8;

/// Tablo başlığını basar.
pub fn print_header() {
    println!(
        "{}",
        format!(
            "{:<COL_SERVICE$}  {:>COL_COUNT$}  {:>COL_COUNT$}  {:>COL_COUNT$}  {:>COL_COUNT$}  {:>COL_COUNT$}",
            "SERVICE", "CRIT", "ERR", "WARN", "NOTICE", "TOTAL"
        )
        .bold()
        .underline()
    );
}

/// Tek bir servis satırını renklendirilmiş olarak basar.
pub fn print_service_row(s: &ServiceSummary) {
    let name = match s.status() {
        HealthStatus::Critical => s.name.bold().red(),
        HealthStatus::Error => s.name.bold().yellow(),
        HealthStatus::Warning => s.name.yellow(),
        HealthStatus::Ok => s.name.normal().into(),
    };

    let crit = if s.critical > 0 {
        format!("{:>COL_COUNT$}", s.critical).red().bold()
    } else {
        format!("{:>COL_COUNT$}", "-").dimmed()
    };

    let err = if s.errors > 0 {
        format!("{:>COL_COUNT$}", s.errors).yellow().bold()
    } else {
        format!("{:>COL_COUNT$}", "-").dimmed()
    };

    let warn = if s.warnings > 0 {
        format!("{:>COL_COUNT$}", s.warnings).cyan()
    } else {
        format!("{:>COL_COUNT$}", "-").dimmed()
    };

    let notice = format!("{:>COL_COUNT$}", s.notices).normal();
    let total = format!("{:>COL_COUNT$}", s.total).normal();

    println!(
        "{:<COL_SERVICE$}  {}  {}  {}  {}  {}",
        name, crit, err, warn, notice, total
    );
}

/// Verbose modda servisin son log satırlarını girintili ve dim olarak basar.
pub fn print_service_detail(s: &ServiceSummary) {
    for log in &s.recent_logs {
        let ts = log
            .timestamp_us
            .map(format_timestamp_us)
            .unwrap_or_else(|| "??:??:??".to_string());

        let msg = truncate_str(log.message.trim(), 120);
        println!("{}", format!("  {}  {}", ts, msg).dimmed());
    }
}

fn format_timestamp_us(ts_us: u64) -> String {
    let ts_s = ts_us / 1_000_000;
    let secs = ts_s % 60;
    let mins = (ts_s / 60) % 60;
    let hours = (ts_s / 3600) % 24;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}…", truncated)
    }
}

/// Özet istatistikleri basar.
pub fn print_footer(summaries: &[ServiceSummary]) {
    let total_services = summaries.len();
    let problematic = summaries
        .iter()
        .filter(|s| s.issue_count() > 0)
        .count();

    println!();
    println!(
        "{}  {} servis tarandı, {} sorunlu",
        "Özet:".bold(),
        total_services,
        if problematic > 0 {
            problematic.to_string().red().bold()
        } else {
            problematic.to_string().green().bold()
        }
    );
}
