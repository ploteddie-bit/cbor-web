use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Telemetry {
    pub scans: u64,
    pub crawls: u64,
    pub first_use: u64,
    pub last_use: u64,
    pub total_seconds: u64,
    pub domains: Vec<String>,
    pub version: String,
}

impl Telemetry {
    fn path() -> PathBuf {
        dirs_next::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".cbor-web-audit")
            .join("telemetry.json")
    }

    pub fn load() -> Self {
        fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Some(p) = Self::path().parent() {
            let _ = fs::create_dir_all(p);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(Self::path(), json);
        }
    }

    pub fn track_scan(&mut self, domain: &str) {
        self.scans += 1;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if self.first_use == 0 { self.first_use = now; }
        self.last_use = now;
        if !self.domains.contains(&domain.to_string()) {
            self.domains.push(domain.to_string());
        }
        self.version = env!("CARGO_PKG_VERSION").to_string();
        self.save();
    }

    pub fn track_crawl(&mut self, domain: &str, elapsed_secs: u64) {
        self.crawls += 1;
        self.total_seconds += elapsed_secs;
        self.track_scan(domain);
    }

    pub fn report(&self) -> serde_json::Value {
        serde_json::json!({
            "scans": self.scans,
            "crawls": self.crawls,
            "unique_domains": self.domains.len(),
            "total_time_hours": self.total_seconds / 3600,
            "first_use": self.first_use,
            "last_use": self.last_use,
            "version": self.version,
        })
    }
}
