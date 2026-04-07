use env_structure_macro::EnvStructure;

mod env_structure {
    pub use crate::*;
}

#[derive(EnvStructure)]
struct Env {
    gcp_project_id: String,

    mobile_socket_addr: std::net::SocketAddr,

    #[env(default = "(default)")]
    firestore_database_name: String,

    // Required. Default value. Validated.
    #[env(validator = validate_refresh_fraction, default = 0.5)]
    google_refresh_fraction: f32,

    secret_phrase: Option<String>,

    #[env(validator = validate_secret_number)]
    secret_number: Option<u32>,

    #[env(nested, required_if("USE_EMULATOR"))]
    emulator_env: Option<EmulatorEnv>,
}

#[derive(EnvStructure)]
struct EmulatorEnv {
    firestore_emulator_host: String,
    firebase_auth_emulator_host: String,
}

fn validate_refresh_fraction(val: &f32) -> Result<(), &'static str> {
    if *val <= 0. {
        return Err("must be greater than 0");
    } else if *val > 1. {
        return Err("must be less than or equal to 1");
    }
    Ok(())
}

fn validate_secret_number(val: &u32) -> Result<(), &'static str> {
    if *val > 10 {
        return Err("must be less than 10");
    }
    Ok(())
}
