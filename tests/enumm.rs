#[derive(env_structure::EnvStructure)]
#[env(key = "FIREBASE_MODE")]
enum FirebaseMode {
    #[env(value = "IN_BETWEEN")]
    InBetween,

    #[env(default, value = "ACTUAL")]
    Actual {
        firebase_credentials_path: std::path::PathBuf,
    },

    #[env(value = "EMULATOR")]
    Emulated(EmulatedFirebaseEnv),
}

#[derive(env_structure::EnvStructure)]
struct EmulatedFirebaseEnv {
    firestore_emulator_host: String,
    firebase_auth_emulator_host: String,
}
