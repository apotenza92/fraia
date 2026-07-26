use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn read_json<T: DeserializeOwned>(file: &Path) -> Result<T> {
    let text = fs::read_to_string(file)?;
    Ok(serde_json::from_str(&text)?)
}

pub fn write_json<T: Serialize>(file: &Path, data: &T) -> Result<()> {
    if let Some(parent) = file.parent() {
        ensure_dir(parent)?;
    }
    let text = serde_json::to_string_pretty(data)? + "\n";
    fs::write(file, text)?;
    Ok(())
}

pub fn timestamp_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}-{}", now.as_secs(), now.subsec_millis())
}

pub fn iso_now() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("unix:{}.{:03}", now.as_secs(), now.subsec_millis())
}

pub fn sum(values: &[f64]) -> f64 {
    values.iter().sum()
}

pub fn max_abs(values: &[f64]) -> f64 {
    values.iter().fold(0.0, |m, v| m.max(v.abs()))
}

pub fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

pub fn format_number(value: f64, digits: usize) -> String {
    format!("{:.*}", digits, value)
}
