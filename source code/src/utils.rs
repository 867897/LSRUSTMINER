
use chrono::{DateTime, Utc};

pub fn parse_login(login: &str) -> (String, Option<String>) {
    if let Some(idx) = login.find('.') {
        (login[..idx].to_string(), Some(login[idx+1..].to_string()))
    } else {
        (login.to_string(), None)
    }
}

pub fn now() -> DateTime<Utc> { Utc::now() }

pub fn normalize_hostport(s: &str) -> String {
    let s = s.trim();
    let schemes = ["stratum+tcp://", "stratum+ssl://", "tcp://", "ssl://", "stratum://"];
    for sch in schemes {
        if let Some(rest) = s.strip_prefix(sch) { return rest.trim_end_matches('/').to_string(); }
    }
    s.trim_end_matches('/').to_string()
}
