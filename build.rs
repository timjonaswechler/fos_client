use std::{
    env,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

fn main() -> io::Result<()> {
    // Keep this simple: ensure the app id file is present in the profile directory
    // where the executable will be emitted (e.g., target/debug or target/release).
    println!("cargo:rerun-if-changed=build.rs");

    let target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target"));
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let profile_dir = target_dir.join(profile);

    let steam_appid_target = profile_dir.join("steam_appid.txt");
    if let Some(p) = steam_appid_target.to_str() {
        println!("cargo:rerun-if-changed={}", p);
    }

    ensure_steam_appid(&profile_dir)
}

fn ensure_steam_appid(profile_dir: &Path) -> io::Result<()> {
    let steam_appid = profile_dir.join("steam_appid.txt");
    let desired = "480\n";

    if let Ok(mut file) = File::open(&steam_appid) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        if contents.trim() == "480" {
            return Ok(());
        }
    }

    fs::create_dir_all(profile_dir)?;
    let mut file = File::create(&steam_appid)?;
    file.write_all(desired.as_bytes())
}
