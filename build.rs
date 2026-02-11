use std::{env, fs, io, path::PathBuf};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=steam_appid.txt"); // Env-fallback

    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let profile_dir = PathBuf::from(target_dir).join(&profile);

    // Steam AppID (aus Env, steam_appid.txt oder default)
    let app_id = match env::var("STEAM_APP_ID") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => {
            // Versuche, eine lokale steam_appid.txt im Crate-Ordner zu lesen
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
            let fallback_path = PathBuf::from(&manifest_dir).join("steam_appid.txt");
            match fs::read_to_string(&fallback_path) {
                Ok(s) => {
                    let id = s.lines().next().unwrap_or("").trim().to_string();
                    if !id.is_empty() {
                        println!(
                            "cargo:warning=Using STEAM_APP_ID from {}",
                            fallback_path.display()
                        );
                        id
                    } else {
                        println!(
                            "cargo:warning=steam_appid.txt is empty; falling back to default app id 480 (Spacewar)"
                        );
                        "480".into()
                    }
                }
                Err(_) => {
                    println!(
                        "cargo:warning=STEAM_APP_ID not set and steam_appid.txt not found; falling back to default app id 480 (Spacewar)"
                    );
                    "480".into()
                }
            }
        }
    };

    let appid_path = profile_dir.join("steam_appid.txt");
    fs::create_dir_all(&profile_dir)?;
    if let Err(e) = fs::write(&appid_path, app_id.as_bytes()) {
        println!(
            "cargo:warning=failed to write {}: {}",
            appid_path.display(),
            e
        );
    }

    // Plattform-spezifisch: Steamworks linken (angenommen SDK in ./steamworks/)
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-search=native=steamworks/redistributable_bin/win64");
        println!("cargo:rustc-link-lib=steam_api64");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-search=native=steamworks/redistributable_linux64");
        println!("cargo:rustc-link-lib=steam_api");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN"); // Neben Binary suchen
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-search=native=steamworks/redistributable_osx32"); // SDK-Pfad anpassen!
        println!("cargo:rustc-link-lib=framework=IOKit"); // Häufig benötigt für Steam
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        // RPATH für dylib (neben .app oder Binary)
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");

        // steam_appid.txt ins Bundle kopieren (für .app)
        let bundle_path = profile_dir.join("Contents/Frameworks/steam_appid.txt");
        fs::create_dir_all(bundle_path.parent().unwrap())?;
        fs::write(bundle_path, app_id.as_bytes())?;
    }

    // Version-Info generieren (aus Cargo.toml oder git)
    // let version = env::var("CARGO_PKG_VERSION").unwrap_or_default();
    // let version_rs = out_dir.join("version.rs");
    // fs::write(
    //     version_rs,
    //     format!("pub const VERSION: &str = \"{version}\";"),
    // )?;

    // Bindings generieren (optional, mit bindgen)
    // println!("cargo:rerun-if-changed=steamworks/sdk/public/steam_api.h");
    // ... bindgen call hier

    Ok(())
}
