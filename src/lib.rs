mod context;
mod from_env;
mod issue;

pub trait EnvStructure: Sized {
    fn parse(ctx: &mut ParseCtx) -> Option<Self>;
}

pub use context::ParseCtx;
pub use env_structure_macro::EnvStructure;
pub use from_env::FromEnv;
pub use issue::ParseIssueKind;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct EnvLoadError;

impl std::fmt::Display for EnvLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to load environment")
    }
}

impl std::error::Error for EnvLoadError {}

pub fn load_and_trace<T: EnvStructure>() -> Result<T, EnvLoadError> {
    let mut ctx = ParseCtx::new();
    let item = T::parse(&mut ctx);
    for inf in &ctx.infos {
        tracing::info!("{}", inf);
    }
    for wrn in &ctx.warnings {
        tracing::warn!("{}", wrn);
    }
    for err in &ctx.errs {
        tracing::error!("{}", err);
    }
    if !ctx.errs.is_empty() {
        return Err(EnvLoadError);
    }
    Ok(item.unwrap())
}
