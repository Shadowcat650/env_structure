use crate::traits::EnvDisplay;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

/// A wrapper structure that displays anything that implements [`EnvDisplay`].
pub struct DisplayWrapper<'a, T: EnvDisplay>(pub &'a T);

impl<'a, T: EnvDisplay> Display for DisplayWrapper<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.display(f)
    }
}

/// A wrapper structure that displays anything that implements [`Debug`].
pub struct DisplayDebugWrapper<'a, T: Debug>(pub &'a T);

impl<'a, T: Debug> Display for DisplayDebugWrapper<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
