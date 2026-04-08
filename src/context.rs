use crate::EnvStructure;
use crate::from_env::{DisplayWrapper, FromEnv};
use crate::issue::ParseIssue;
use std::env;
use std::fmt::Display;

/// Context that helps parse and report things during environment loading.
pub struct ParseCtx<'a> {
    pub(crate) errs: Vec<ParseIssue<'a>>,
    pub(crate) warnings: Vec<ParseIssue<'a>>,
    pub(crate) infos: Vec<ParseIssue<'a>>,
}

impl<'a> ParseCtx<'a> {
    pub fn new() -> Self {
        Self {
            errs: Vec::new(),
            warnings: Vec::new(),
            infos: Vec::new(),
        }
    }

    /// Returns `true` if the context found an error.
    pub fn has_errors(&self) -> bool {
        !self.errs.is_empty()
    }

    /// Merges this context's reports with another's.
    pub fn merge(&mut self, other: Self) {
        self.errs.extend(other.errs);
        self.warnings.extend(other.warnings);
        self.infos.extend(other.infos);
    }

    /// Parses a nested [`EnvStruct`].
    pub fn parse_nested<T: EnvStructure>(&mut self) -> Option<T> {
        let mut child = Self::new();
        let nested = T::parse(&mut child);
        self.merge(child);
        nested
    }

    /// Parses a nested [`EnvStruct`] if the condition env variable is `true`.
    pub fn parse_nested_if<T: EnvStructure>(&mut self, cond: &'a str) -> Option<T> {
        let cond = self.parse_with_default(cond, || false);
        if cond {
            let mut child = Self::new();
            let nested = T::parse(&mut child);
            self.merge(child);
            return nested;
        }
        None
    }

    /// Parses an environment value.
    pub fn parse<T: FromEnv>(&mut self, key: &'a str, optional: bool) -> Option<T> {
        match T::parse(env::var(key)) {
            Ok(val) => Some(val),
            Err(issue_kind) => {
                if issue_kind.is_not_found() && optional {
                    return None;
                }
                self.errs.push(ParseIssue::new(key, issue_kind));
                None
            }
        }
    }

    /// Parses an environment value and ensures it is valid.
    pub fn parse_validated<T: FromEnv, V, E>(
        &mut self,
        key: &'a str,
        validate: V,
        optional: bool,
    ) -> Option<T>
    where
        V: Fn(&T) -> Result<(), E>,
        E: Display,
    {
        self.parse(key, optional)
            .and_then(|val| match validate(&val) {
                Ok(_) => Some(val),
                Err(msg) => {
                    self.errs
                        .push(ParseIssue::invalid_value(key, DisplayWrapper(&val), msg));
                    None
                }
            })
    }

    /// Parses an environment value and inserts a default one if it is missing or invalid.
    pub fn parse_with_default<T: FromEnv, D>(&mut self, key: &'a str, default: D) -> T
    where
        D: FnOnce() -> T,
    {
        T::parse(env::var(key)).unwrap_or_else(|issue_kind| {
            let default = default();
            let issue = ParseIssue::new(key, issue_kind)
                .with_recovery(DisplayWrapper(&default).to_string());
            if issue.kind.is_not_found() {
                // It's not an error to have a missing value with a default.
                self.infos.push(issue);
            } else {
                self.warnings.push(issue);
            }
            default
        })
    }

    /// Parses and validates an environment value and inserts a default if its missing or invalid.
    pub fn parse_validated_with_default<T, V, E, D>(
        &mut self,
        key: &'a str,
        validate: V,
        default: D,
    ) -> T
    where
        T: FromEnv,
        V: Fn(&T) -> Result<(), E>,
        E: Display,
        D: FnOnce() -> T,
    {
        match T::parse(env::var(key)) {
            Ok(val) => match validate(&val) {
                Ok(_) => val,
                Err(msg) => {
                    let issue = ParseIssue::invalid_value(key, DisplayWrapper(&val), msg);
                    self.report_and_default(issue, validate, default)
                }
            },
            Err(issue_kind) => {
                self.report_and_default(ParseIssue::new(key, issue_kind), validate, default)
            }
        }
    }

    /// Properly reports and sets a default value of a validated environment var.
    pub fn report_and_default<T, V, E, D>(
        &mut self,
        issue: ParseIssue<'a>,
        validate: V,
        default: D,
    ) -> T
    where
        T: FromEnv,
        V: Fn(&T) -> Result<(), E>,
        E: Display,
        D: FnOnce() -> T,
    {
        let default = default();
        if let Err(msg) = validate(&default) {
            let recovery = format!(
                "default value '{}' is invalid: {msg}",
                DisplayWrapper(&default)
            );
            self.errs.push(issue.with_recovery(recovery));
        } else {
            let recovery = format!("defaulting to '{}", DisplayWrapper(&default));
            if issue.kind.is_not_found() {
                // It's not an error to have a missing value with a default.
                self.infos.push(issue.with_recovery(recovery));
            } else {
                self.warnings.push(issue.with_recovery(recovery));
            }
        }
        default
    }
}
