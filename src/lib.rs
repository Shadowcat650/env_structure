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
