use crate::utils::DisplayDebugWrapper;
use std::fmt::{Display, Formatter};

/// The issue with the environment variable.
pub enum ParseIssueKind {
    NotFound,
    InvalidValue { value: String, msg: String },
}

impl ParseIssueKind {
    pub fn invalid_value(value: impl Display, msg: impl Display) -> Self {
        Self::InvalidValue {
            value: value.to_string(),
            msg: msg.to_string(),
        }
    }

    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound)
    }
}

impl From<std::env::VarError> for ParseIssueKind {
    fn from(err: std::env::VarError) -> Self {
        match err {
            std::env::VarError::NotPresent => Self::NotFound,
            std::env::VarError::NotUnicode(x) => {
                Self::invalid_value(DisplayDebugWrapper(&x), "not valid unicode")
            }
        }
    }
}

/// An issue with an environment value.
pub(crate) struct ParseIssue<'a> {
    /// The variable key.
    var: &'a str,

    /// The kind of issue.
    pub(crate) kind: ParseIssueKind,

    /// How the issue was attempted (successfully or not) to be resolved.
    recovery: Option<String>,
}

impl<'a> ParseIssue<'a> {
    pub fn new(var: &'a str, kind: ParseIssueKind) -> Self {
        Self {
            var,
            kind,
            recovery: None,
        }
    }

    pub fn invalid_value(var: &'a str, val: String, msg: String) -> Self {
        Self {
            var,
            kind: ParseIssueKind::InvalidValue { value: val, msg },
            recovery: None,
        }
    }

    pub fn with_recovery(mut self, recovery: String) -> Self {
        self.recovery = Some(recovery);
        self
    }
}

impl<'a> Display for ParseIssue<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ParseIssueKind::NotFound => {
                write!(f, "{} not found in environment", self.var)?;
            }
            ParseIssueKind::InvalidValue { value, msg } => {
                write!(f, "{}={}; {}", self.var, value, msg)?;
            }
        }
        if let Some(recovery) = &self.recovery {
            write!(f, ": {}", recovery)?;
        }
        Ok(())
    }
}
