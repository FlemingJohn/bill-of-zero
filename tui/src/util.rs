//! Small helpers with no dependencies.

use anyhow::{bail, Result};

/// Convert a `YYYY-MM-DD` date (UTC midnight) to a Unix timestamp.
/// Uses Howard Hinnant's days-from-civil algorithm — no chrono needed.
pub fn ymd_to_unix(s: &str) -> Result<u64> {
    let parts: Vec<&str> = s.trim().split('-').collect();
    if parts.len() != 3 {
        bail!("date must be YYYY-MM-DD");
    }
    let y: i64 = parts[0].parse().map_err(|_| anyhow::anyhow!("bad year"))?;
    let m: i64 = parts[1].parse().map_err(|_| anyhow::anyhow!("bad month"))?;
    let d: i64 = parts[2].parse().map_err(|_| anyhow::anyhow!("bad day"))?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        bail!("month/day out of range");
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = (y - era * 400) as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Ok((days * 86400) as u64)
}

/// Format a stroop balance string with thousands separators (best-effort).
pub fn group(n: &str) -> String {
    let n = n.trim();
    let neg = n.starts_with('-');
    let digits: String = n.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return n.to_string();
    }
    let mut out = String::new();
    for (i, c) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    let grouped: String = out.chars().rev().collect();
    if neg {
        format!("-{grouped}")
    } else {
        grouped
    }
}
