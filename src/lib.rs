mod context;
mod from_env;
mod issue;

pub trait EnvStructure: Sized {
    fn parse(ctx: &mut ParseCtx) -> Option<Self>;
}

pub use context::ParseCtx;
pub use from_env::FromEnv;
pub use issue::ParseIssueKind;

// impl EnvStruct for Env {
//     fn parse(ctx: &mut ParseCtx) -> Option<Self> {
//         let gcp_project_id = ctx.parse("GCP_PROJECT_ID", false);
//         let mobile_socket_addr = ctx.parse("MOBILE_SOCKET_ADDR", false);
//         let firestore_database_name = ctx.parse_with_default("FIRESTORE_DATABASE_NAME", || "(default)".into());
//         let google_refresh_fraction = ctx.parse_validated_with_default("GOOGLE_REFRESH_FRACTION", validate_refresh_fraction, || 0.5);
//         let secret_phrase = ctx.parse("SECRET_PHRASE", true);
//         let secret_number = ctx.parse_validated("SECRET_NUMBER", validate_secret_number, true);
//         let emulator_env = ctx.parse_nested_if("USE_EMULATOR");
//         if ctx.has_errors() {
//             return None;
//         }
//         Some(Self {
//             gcp_project_id: gcp_project_id.unwrap(),
//             mobile_socket_addr: mobile_socket_addr.unwrap(),
//             firestore_database_name,
//             google_refresh_fraction,
//             secret_phrase,
//             secret_number,
//             emulator_env,
//         })
//     }
// }

// impl EnvStruct for EmulatorEnv {
//     fn parse(ctx: &mut ParseCtx) -> Option<Self> {
//         let firestore_emulator_host = ctx.parse("FIRESTORE_EMULATOR_HOST", false);
//         let firebase_auth_emulator_host = ctx.parse("FIREBASE_AUTH_EMULATOR_HOST", false);
//         if ctx.has_errors() {
//             return None;
//         }
//         Some(Self {
//             firestore_emulator_host: firestore_emulator_host.unwrap(),
//             firebase_auth_emulator_host: firebase_auth_emulator_host.unwrap(),
//         })
//     }
// }

// impl ::env_structure::EnvStructure for EmulatorEnv {
//     fn parse(ctx: &mut ::env_structure::ParseCtx) -> ::std::option::Option<Self> {
//         let firestore_emulator_host = ctx.parse("FIRESTORE_EMULATOR_HOST", false);
//         let firebase_auth_emulator_host = ctx.parse("FIREBASE_AUTH_EMULATOR_HOST", false);
//         if ctx.has_errors() { return ::std::option::Option::None; }
//         ::std::option::Option::Some(Self { firestore_emulator_host, firebase_auth_emulator_host })
//     }
// }
