use crate::issue::ParseIssueKind;
use std::env::VarError;
use std::fmt::Display;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
};
use std::path::PathBuf;
use std::str::FromStr;

pub trait FromEnv: Sized + Display {
    fn parse(input: Result<String, VarError>) -> Result<Self, ParseIssueKind>;
}

impl FromEnv for bool {
    fn parse(input: Result<String, VarError>) -> Result<Self, ParseIssueKind> {
        let val = input?;
        match val.as_str() {
            "true" | "TRUE" | "yes" | "YES" | "1" => Ok(true),
            "false" | "FALSE" | "no" | "NO" | "0" => Ok(false),
            _ => Err(ParseIssueKind::invalid_value("not valid boolean")),
        }
    }
}

// Automatic implementations.

trait ParseFromStr {}

impl<T: FromStr + Display + ParseFromStr> FromEnv for T
where
    <T as FromStr>::Err: Display,
{
    fn parse(input: Result<String, VarError>) -> Result<Self, ParseIssueKind> {
        let val = input?;
        T::from_str(&val).map_err(|err| ParseIssueKind::invalid_value(err))
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
