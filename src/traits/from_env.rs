use crate::ParseIssueKind;
use crate::traits::env_display::EnvDisplay;
use std::env;
use std::fmt::Display;
use std::str::FromStr;

use std::ffi::OsString;

pub struct InvalidValueError {
    value: RawInput,
    msg: String,
}

impl InvalidValueError {
    pub fn new(value: impl Into<RawInput>, msg: impl Display) -> Self {
        Self {
            value: value.into(),
            msg: msg.to_string(),
        }
    }
}

pub struct FromEnvCtx {
    secret: bool,
    input: Result<String, env::VarError>,
}

pub enum RawInput {
    String(String),
    OsString(OsString),
}

impl From<String> for RawInput {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<OsString> for RawInput {
    fn from(value: OsString) -> Self {
        Self::OsString(value)
    }
}

impl FromEnvCtx {
    pub(crate) fn new(key: &str, secret: bool) -> Self {
        Self {
            secret,
            input: env::var(key),
        }
    }

    pub(crate) fn parse<T: FromEnv>(self) -> Result<T, ParseIssueKind> {
        T::parse(self).0
    }

    pub fn parse_from_str<P, T>(self, parser: P) -> FromEnvResult<T>
    where
        P: Fn(String) -> Result<T, InvalidValueError>,
    {
        self.parse_from_raw(|raw| {
            let input = match raw {
                RawInput::String(val) => val,
                RawInput::OsString(val) => {
                    return Err(InvalidValueError {
                        value: RawInput::OsString(val),
                        msg: "not valid unicode".to_string(),
                    });
                }
            };
            parser(input)
        })
    }

    pub fn parse_from_raw<P, T>(self, parser: P) -> FromEnvResult<T>
    where
        P: Fn(RawInput) -> Result<T, InvalidValueError>,
    {
        let input = match self.input {
            Ok(val) => RawInput::String(val),
            Err(env::VarError::NotUnicode(val)) => RawInput::OsString(val),
            Err(env::VarError::NotPresent) => return FromEnvResult(Err(ParseIssueKind::NotFound)),
        };

        FromEnvResult(parser(input).map_err(|err| ParseIssueKind::InvalidValue {
            value: Self::safe_value_str(err.value, self.secret),
            msg: err.msg,
        }))
    }

    fn safe_value_str(value: RawInput, secret: bool) -> String {
        if secret {
            return "<REDACTED>".into();
        }
        match value {
            RawInput::String(val) => val,
            RawInput::OsString(val) => format!("{:?}", val),
        }
    }
}

pub struct FromEnvResult<T>(Result<T, ParseIssueKind>);

pub trait FromEnv: Sized {
    fn parse(ctx: FromEnvCtx) -> FromEnvResult<Self>;
}

impl FromEnv for bool {
    fn parse(ctx: FromEnvCtx) -> FromEnvResult<Self> {
        ctx.parse_from_str(|val| match val.as_str() {
            "true" | "TRUE" | "yes" | "YES" | "1" => Ok(true),
            "false" | "FALSE" | "no" | "NO" | "0" => Ok(false),
            _ => Err(InvalidValueError::new(val, "not a valid boolean")),
        })
    }
}

impl FromEnv for OsString {
    fn parse(ctx: FromEnvCtx) -> FromEnvResult<Self> {
        ctx.parse_from_raw(|input| match input {
            RawInput::String(val) => Ok(val.into()),
            RawInput::OsString(val) => Ok(val),
        })
    }
}

impl FromEnv for std::path::PathBuf {
    fn parse(ctx: FromEnvCtx) -> FromEnvResult<Self> {
        ctx.parse_from_raw(|input| match input {
            RawInput::String(val) => Ok(val.into()),
            RawInput::OsString(val) => Ok(val.into()),
        })
    }
}

// Automatic implementations.

impl<T: FromStr + EnvDisplay + ParseFromStr> FromEnv for T
where
    <T as FromStr>::Err: Display,
{
    fn parse(ctx: FromEnvCtx) -> FromEnvResult<Self> {
        ctx.parse_from_str(|val| T::from_str(&val).map_err(|err| InvalidValueError::new(val, err)))
    }
}

/// Select objects that can be parsed from a string.
trait ParseFromStr {}

macro_rules! impl_from_env_via_fromstr {
    ($($t:ty),*) => {
        $(impl ParseFromStr for $t {})*
    };
}

impl_from_env_via_fromstr!(
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64,
    String,
    std::net::IpAddr,
    std::net::Ipv4Addr,
    std::net::Ipv6Addr,
    std::net::SocketAddr,
    std::net::SocketAddrV4,
    std::net::SocketAddrV6,
    std::num::NonZeroI8,
    std::num::NonZeroI16,
    std::num::NonZeroI32,
    std::num::NonZeroI64,
    std::num::NonZeroI128,
    std::num::NonZeroIsize,
    std::num::NonZeroU8,
    std::num::NonZeroU16,
    std::num::NonZeroU32,
    std::num::NonZeroU64,
    std::num::NonZeroU128,
    std::num::NonZeroUsize
);
