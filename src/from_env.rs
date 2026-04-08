use crate::issue::ParseIssueKind;
use std::env::VarError;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
};
use std::path::PathBuf;
use std::str::FromStr;

pub trait FromEnv: EnvDisplay + Sized {
    fn parse(input: Result<String, VarError>) -> Result<Self, ParseIssueKind>;
}

pub(crate) trait EnvDisplay {
    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl FromEnv for bool {
    fn parse(input: Result<String, VarError>) -> Result<Self, ParseIssueKind> {
        let val = input?;
        match val.as_str() {
            "true" | "TRUE" | "yes" | "YES" | "1" => Ok(true),
            "false" | "FALSE" | "no" | "NO" | "0" => Ok(false),
            _ => Err(ParseIssueKind::invalid_value(val, "not valid boolean")),
        }
    }
}

impl EnvDisplay for PathBuf {
    fn display(&self, f: &mut Formatter<'_>) -> fmt::Result {
        DisplayDebugWrapper(self).fmt(f)
    }
}

// Automatic implementations.

trait ParseFromStr {}

impl<T: FromStr + EnvDisplay + ParseFromStr> FromEnv for T
where
    <T as FromStr>::Err: Display,
{
    fn parse(input: Result<String, VarError>) -> Result<Self, ParseIssueKind> {
        let val = input?;
        T::from_str(&val).map_err(|err| ParseIssueKind::invalid_value(val, err))
    }
}

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
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
    SocketAddr,
    SocketAddrV4,
    SocketAddrV6,
    PathBuf,
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroIsize,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroUsize
);

trait ParseFromDisplay {}

impl<T: Display + ParseFromDisplay> EnvDisplay for T {
    fn display(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <T as Display>::fmt(self, f)
    }
}

macro_rules! impl_env_display_via_display {
    ($($t:ty),*) => {
        $(impl ParseFromDisplay for $t {})*
    };
}

impl_env_display_via_display!(
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
    bool,
    String,
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
    SocketAddr,
    SocketAddrV4,
    SocketAddrV6,
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroIsize,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroUsize
);

pub struct DisplayWrapper<'a, T: EnvDisplay>(pub &'a T);

impl<'a, T: EnvDisplay> Display for DisplayWrapper<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.display(f)
    }
}

pub struct DisplayDebugWrapper<'a, T: Debug>(pub &'a T);
impl<'a, T: Debug> Display for DisplayDebugWrapper<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
