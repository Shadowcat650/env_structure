use crate::ParseCtx;

/// An object that can be loaded from the environment.
pub trait EnvStructure: Sized {
    fn parse(ctx: &mut ParseCtx) -> Option<Self>;
}
