use std::collections::{HashMap, VecDeque};

use crate::journal::JournalEntry;

const MAX_RECENT_LOGS: usize = 5;

#[derive(Debug)]
pub struct LogLine {
    pub timestamp_us: Option<u64>,
    pub message: String,
}

/// Tek bir servis için hata/uyarı özeti.
#[derive(Debug, Default)]
pub struct ServiceSummary {
    pub name: String,
    pub critical: u32, // priority 0-2 (emerg/alert/crit)
    pub errors: u32,   // priority 3
    pub warnings: u32, // priority 4
    pub notices: u32,  // priority 5
    pub total: u32,    // toplam entry sayısı
    pub recent_logs: VecDeque<LogLine>, // son MAX_RECENT_LOGS log satırı
}

impl ServiceSummary {
    /// Kritik + hata + uyarı toplamı; sıralama için kullanılır.
    pub fn issue_count(&self) -> u32 {
        self.critical + self.errors + self.warnings
    }

    /// Servisin genel durumu.
    pub fn status(&self) -> HealthStatus {
        if self.critical > 0 {
            HealthStatus::Critical
        } else if self.errors > 0 {
            HealthStatus::Error
        } else if self.warnings > 0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Ok
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum HealthStatus {
    Critical,
    Error,
    Warning,
    Ok,
}

/// Entry listesini servislere göre gruplar ve özetler.
/// Sonuç, issue_count'a göre azalan sırada döner.
pub fn summarize(entries: &[JournalEntry]) -> Vec<ServiceSummary> {
    let mut map: HashMap<String, ServiceSummary> = HashMap::new();

    for entry in entries {
        let name = entry.service_name();
        let summary = map.entry(name.clone()).or_insert_with(|| ServiceSummary {
            name,
            ..Default::default()
        });

        summary.total += 1;

        let prio = entry.priority_level();
        match prio {
            0..=2 => summary.critical += 1,
            3 => summary.errors += 1,
            4 => summary.warnings += 1,
            5 => summary.notices += 1,
            _ => {}
        }

        // Son MAX_RECENT_LOGS log satırını kayan pencere ile tut
        if summary.recent_logs.len() >= MAX_RECENT_LOGS {
            summary.recent_logs.pop_front();
        }
        summary.recent_logs.push_back(LogLine {
            timestamp_us: entry
                .realtime_timestamp
                .as_deref()
                .and_then(|s| s.parse().ok()),
            message: entry.message_text(),
        });
    }

    let mut summaries: Vec<ServiceSummary> = map.into_values().collect();
    summaries.sort_by(|a, b| b.issue_count().cmp(&a.issue_count()));
    summaries
}
