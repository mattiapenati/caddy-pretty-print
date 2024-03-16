use anyhow::{Context, Result};

use crate::record::LogRecord;

#[derive(Default)]
pub struct FiltersBuilder {
    strict: bool,
    host_patterns: Vec<glob::Pattern>,
}

impl FiltersBuilder {
    pub fn with_strict(&mut self, strict: bool) -> &mut Self {
        self.strict = strict;
        self
    }

    pub fn with_host(&mut self, host: &str) -> Result<&mut Self> {
        let pattern =
            glob::Pattern::new(host).with_context(|| format!("invalid host filter: {}", host))?;
        self.host_patterns.push(pattern);
        Ok(self)
    }

    pub fn build(self) -> Result<Filters> {
        Ok(Filters {
            strict: self.strict,
            host_patterns: self.host_patterns,
        })
    }
}

pub struct Filters {
    strict: bool,
    host_patterns: Vec<glob::Pattern>,
}

impl Filters {
    pub fn builder() -> FiltersBuilder {
        FiltersBuilder::default()
    }

    pub fn is_strict(&self) -> bool {
        self.strict
    }

    pub fn matches(&self, record: &LogRecord) -> bool {
        self.matches_host(record)
    }

    fn matches_host(&self, record: &LogRecord) -> bool {
        if self.host_patterns.is_empty() {
            return true;
        };
        let Some(host) = record.request.as_ref().map(|req| req.host.as_str()) else {
            return false;
        };

        self.host_patterns
            .iter()
            .any(|pattern| pattern.matches(host))
    }
}
