use std::{env, fs, io, path::PathBuf};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set by Cargo");

    // STEAM_APP_ID embedded (deine Env)
    let appid_str = env::var("STEAM_APP_ID").unwrap_or_else(|_| {
        println!("cargo:warning=STEAM_APP_ID missing → dev 480");
        "480".to_string()
    });

    // config.rs generieren
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let is_release = profile == "release";
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string());

    let config = format!(
        r#"pub const STEAM_APP_ID: u32 = {};
pub const IS_RELEASE: bool = {};
pub const BUILD_PROFILE: &str = "{}";
pub const VERSION: &str = "{}";"#,
        appid_str, is_release, profile, version
    );
    fs::write(PathBuf::from(&out_dir).join("config.rs"), config)?;

    Ok(())
}
