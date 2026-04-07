use std::fmt::Display;

/// The issue with the environment variable.
pub enum ParseIssueKind {
    NotFound,
    InvalidValue(String),
}

impl ParseIssueKind {
    pub fn invalid_value(msg: impl Display) -> Self {
        Self::InvalidValue(msg.to_string())
    }

    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound)
    }
}

impl From<std::env::VarError> for ParseIssueKind {
    fn from(err: std::env::VarError) -> Self {
        match err {
            std::env::VarError::NotPresent => Self::NotFound,
            std::env::VarError::NotUnicode(_) => Self::invalid_value("not valid unicode"),
        }
    }
}

/// An issue with an environment value.
pub(crate) struct ParseIssue<'a> {
    /// The variable key.
    pub var: &'a str,

    /// The kind of issue.
    pub kind: ParseIssueKind,

    /// How the issue was attempted (successfully or not) to be resolved.
    pub recovery: Option<String>,
}

impl<'a> ParseIssue<'a> {
    pub fn new(var: &'a str, kind: ParseIssueKind) -> Self {
        Self {
            var,
            kind,
            recovery: None,
        }
    }

    pub fn invalid_value(var: &'a str, msg: impl Display) -> Self {
        Self {
            var,
            kind: ParseIssueKind::InvalidValue(msg.to_string()),
            recovery: None,
        }
    }

    pub fn with_recovery(mut self, recovery: String) -> Self {
        self.recovery = Some(recovery);
        self
    }
}
