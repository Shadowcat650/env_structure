use crate::utils::DisplayDebugWrapper;
use std::fmt;
use std::fmt::{Display, Formatter};

pub trait EnvDisplay {
    fn display(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

impl EnvDisplay for std::path::PathBuf {
    fn display(&self, f: &mut Formatter<'_>) -> fmt::Result {
        DisplayDebugWrapper(self).fmt(f)
    }
}

impl EnvDisplay for std::ffi::OsString {
    fn display(&self, f: &mut Formatter<'_>) -> fmt::Result {
        DisplayDebugWrapper(self).fmt(f)
    }
}

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
