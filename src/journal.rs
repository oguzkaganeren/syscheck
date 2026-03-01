use serde::Deserialize;

/// journalctl -o json çıktısındaki tek bir log satırı.
/// Alanlar opsiyonel çünkü her entry her alanı içermeyebilir.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct JournalEntry {
    #[serde(rename = "PRIORITY")]
    pub priority: Option<String>,

    #[serde(rename = "_SYSTEMD_UNIT")]
    pub systemd_unit: Option<String>,

    #[serde(rename = "SYSLOG_IDENTIFIER")]
    pub syslog_identifier: Option<String>,

    /// MESSAGE bazen string, bazen byte array olarak gelir.
    #[serde(rename = "MESSAGE")]
    pub message: Option<serde_json::Value>,

    #[serde(rename = "__REALTIME_TIMESTAMP")]
    pub realtime_timestamp: Option<String>,

    #[serde(rename = "_HOSTNAME")]
    pub hostname: Option<String>,
}

impl JournalEntry {
    /// Önce systemd unit adını, yoksa syslog identifier'ı döner.
    pub fn service_name(&self) -> String {
        self.systemd_unit
            .clone()
            .or_else(|| self.syslog_identifier.clone())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// PRIORITY alanını u8'e çevirir; parse edilemezse Info (6) kabul eder.
    pub fn priority_level(&self) -> u8 {
        self.priority
            .as_deref()
            .and_then(|p| p.parse().ok())
            .unwrap_or(6)
    }

    /// MESSAGE alanını human-readable string'e çevirir.
    pub fn message_text(&self) -> String {
        match &self.message {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(serde_json::Value::Array(bytes)) => {
                // byte array → UTF-8 string
                let raw: Vec<u8> = bytes
                    .iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u8))
                    .collect();
                String::from_utf8_lossy(&raw).into_owned()
            }
            _ => String::new(),
        }
    }
}

/// `journalctl -o json` çıktısını (her satır bir JSON nesnesi) parse eder.
/// Parse edilemeyen satırları sessizce atlar.
pub fn parse_journal_lines(input: &str) -> Vec<JournalEntry> {
    input
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}
