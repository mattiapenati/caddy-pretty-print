use std::net::{IpAddr, SocketAddr};

use colored::Colorize;
use serde::Deserialize;
use serde_with::{serde_as, DefaultOnError, DeserializeAs, DisplayFromStr};
use terminal_size::{terminal_size, Width};
use time::OffsetDateTime;

#[serde_as]
#[derive(Deserialize)]
pub struct LogRecord {
    #[serde(rename = "ts")]
    timestamp: f64,
    level: LogLevel,
    #[serde(rename = "msg")]
    message: String,
    pub request: Option<LogRequest>,
    duration: Option<f64>,
    #[serde(default)]
    #[serde_as(as = "DefaultOnError<Option<SerdeHttpStatusCode>>")]
    status: Option<http::StatusCode>,
}

#[serde_as]
#[derive(Deserialize)]
pub struct LogRequest {
    remote_ip: IpAddr,
    #[serde_as(as = "DisplayFromStr")]
    remote_port: u16,
    #[serde(with = "http_serde::method")]
    method: http::Method,
    pub host: String,
    uri: String,
    #[serde(rename = "proto", with = "http_serde::version")]
    version: http::Version,
    #[serde(with = "http_serde::header_map")]
    headers: http::HeaderMap,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Panic,
    Fatal,
}

impl LogRecord {
    const TIMESTAMP: &'static [time::format_description::FormatItem<'static>] = time::macros::format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:6][offset_hour sign:mandatory]:[offset_minute]"
    );

    pub fn format(self) -> String {
        let timestamp = Self::format_timestamp(self.timestamp);
        let level = Self::format_level(self.level);
        let indent = 4;
        let message = self
            .request
            .map(|req| Self::format_request(req, indent))
            .unwrap_or_else(|| self.message);
        let mut lines = vec![format!("[{timestamp}] {level} {message}")];
        if let Some(status) = self.status {
            lines.push(format!(
                "{:indent$}status          {}",
                ' ',
                Self::format_status(status),
                indent = indent
            ));
        }
        if let Some(duration) = self.duration {
            lines.push(format!(
                "{:indent$}duration        {}",
                ' ',
                Self::format_duration(duration),
                indent = indent
            ));
        }
        lines.join("\n")
    }

    fn format_timestamp(ts: f64) -> String {
        let ts = (ts * 1_000_000.0) as i128 * 1_000;
        let ts = OffsetDateTime::from_unix_timestamp_nanos(ts).unwrap();
        ts.format(&Self::TIMESTAMP).unwrap()
    }

    fn format_level(level: LogLevel) -> String {
        match level {
            LogLevel::Debug => "DEBUG".yellow(),
            LogLevel::Info => " INFO".cyan(),
            LogLevel::Warn => " WARN".magenta(),
            LogLevel::Error => "ERROR".red(),
            LogLevel::Panic => "PANIC".reversed(),
            LogLevel::Fatal => "FATAL".reversed(),
        }
        .to_string()
    }

    fn format_request(request: LogRequest, indent: usize) -> String {
        let mut lines = vec![format!(
            "{} {} {:?}",
            request.method, request.uri, request.version
        )];

        let remote_addr = SocketAddr::from((request.remote_ip, request.remote_port));
        lines.push(format!(
            "{:indent$}remote address  {}",
            "",
            remote_addr,
            indent = indent
        ));
        lines.push(format!(
            "{:indent$}host            {}",
            "",
            request.host,
            indent = indent
        ));
        if let Some(user_agent) = request
            .headers
            .get(http::header::USER_AGENT)
            .and_then(|h| h.to_str().ok())
        {
            lines.push(format!(
                "{:indent$}user-agent      {}",
                "",
                user_agent,
                indent = indent
            ));
        }

        if let Some((Width(width), _)) = terminal_size() {
            lines.iter_mut().for_each(|line| truncate_line(line, width));
        }
        lines.join("\n")
    }

    fn format_status(status: http::StatusCode) -> String {
        let code = if status.is_informational() || status.is_success() {
            status.as_u16().to_string().green().to_string()
        } else if status.is_redirection() {
            status.as_u16().to_string().cyan().to_string()
        } else if status.is_server_error() || status.is_client_error() {
            status.as_u16().to_string().red().to_string()
        } else {
            status.as_u16().to_string()
        };

        match status.canonical_reason() {
            Some(reason) => format!("{} {}", code, reason),
            None => code,
        }
    }

    fn format_duration(duration: f64) -> String {
        if duration * 1_000.0 < 1.0 {
            let micros = duration * 1_000_000.0;
            format!("{:.03} us", micros)
        } else if duration < 1.0 {
            let millis = duration * 1_000.0;
            format!("{:.03} ms", millis)
        } else if duration < 60.0 {
            format!("{:.03} s", duration)
        } else {
            let minutes = duration.div_euclid(60.0).floor() as u64;
            let seconds = duration.rem_euclid(60.0);
            format!("{} m {:.03} s", minutes, seconds)
        }
    }
}

fn truncate_line(str: &mut String, width: u16) {
    let width = width as usize;
    if str.len() + 1 > width {
        *str = str
            .chars()
            .take(width.saturating_sub(2))
            .chain(['…'])
            .collect();
    }
}

struct SerdeHttpStatusCode;

impl<'de> DeserializeAs<'de, http::StatusCode> for SerdeHttpStatusCode {
    fn deserialize_as<D>(deserializer: D) -> Result<http::StatusCode, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        http_serde::status_code::deserialize(deserializer)
    }
}
