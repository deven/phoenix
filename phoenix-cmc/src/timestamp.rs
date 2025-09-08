use chrono::{DateTime, Duration, Local, TimeZone};
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Timestamp {
    pub time: DateTime<Local>,
}

impl Timestamp {
    pub const MAX_FORMATTED_LENGTH: usize = 24;

    pub fn new() -> Self {
        Self { time: Local::now() }
    }

    pub fn from_unix(timestamp: i64) -> Self {
        Self { time: Local.timestamp_opt(timestamp, 0).unwrap() }
    }

    pub fn unix(&self) -> i64 {
        self.time.timestamp()
    }

    pub fn date(&self, start: usize, len: usize) -> String {
        let formatted = self.time.format("%a %b %e %T %Y").to_string();
        let formatted = if formatted.len() > Self::MAX_FORMATTED_LENGTH { &formatted[..Self::MAX_FORMATTED_LENGTH] } else { &formatted };

        if len > 0 && start + len < formatted.len() {
            formatted[start..start + len].to_string()
        } else if start < formatted.len() {
            formatted[start..].to_string()
        } else {
            String::new()
        }
    }

    pub fn stamp(&self) -> String {
        let now = Timestamp::new();

        // Check for different year or future timestamp
        let now_year = now.time.format("%Y").to_string();
        let self_year = self.time.format("%Y").to_string();

        if self.time > now.time || now_year != self_year {
            // Different year or future timestamp, return "Mmm dd yyyy hh:mm" format
            let month_day = self.date(4, 7);
            let year = self.date(20, 4);
            let time = self.date(10, 6);
            format!("{month_day} {year} {time}")
        } else {
            // Check for different week
            let lastweek = now.time - Duration::days(7);
            let lastweek_date = lastweek.format("%b %e").to_string();
            let self_date = self.time.format("%b %e").to_string();

            if self.time < lastweek && lastweek_date != self_date {
                // Same year, not in past week, return "Mmm dd hh:mm" format
                self.date(4, 12)
            } else {
                // Check for different day
                let now_date = now.time.format("%b %e").to_string();
                if now_date != self_date {
                    // Different day, within past week, return "Ddd hh:mm" format
                    let day = self.date(0, 4);
                    let time = self.date(11, 5);
                    format!("{day} {time}")
                } else {
                    // Same day, return "hh:mm" format
                    self.date(11, 5)
                }
            }
        }
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{date}", date = self.date(0, 0))
    }
}

impl From<i64> for Timestamp {
    fn from(timestamp: i64) -> Self {
        if timestamp == 0 {
            Self::new()
        } else {
            Self::from_unix(timestamp)
        }
    }
}

impl std::ops::Sub for Timestamp {
    type Output = i64;

    fn sub(self, other: Self) -> Self::Output {
        self.unix() - other.unix()
    }
}

impl std::ops::Sub<i64> for Timestamp {
    type Output = Timestamp;

    fn sub(self, seconds: i64) -> Self::Output {
        Timestamp::from_unix(self.unix() - seconds)
    }
}

impl std::ops::Add<i64> for Timestamp {
    type Output = Timestamp;

    fn add(self, seconds: i64) -> Self::Output {
        Timestamp::from_unix(self.unix() + seconds)
    }
}

impl PartialEq for Timestamp {
    fn eq(&self, other: &Self) -> bool {
        self.unix() == other.unix()
    }
}

impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.unix().partial_cmp(&other.unix())
    }
}

impl Eq for Timestamp {}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.unix().cmp(&other.unix())
    }
}

// Utility function to get system uptime if available.
pub async fn system_uptime() -> Option<i64> {
    #[cfg(target_os = "linux")]
    {
        tokio::fs::read_to_string("/proc/uptime")
            .await
            .ok()
            .and_then(|content| content.split_whitespace().next().and_then(|s| s.parse::<f64>().ok()).map(|f| f as i64))
    }

    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}
