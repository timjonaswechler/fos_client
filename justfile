# =============================================================================
# fos_client: Clean Build Automation
# =============================================================================
# Usage:
#   just                          # Hilfe
#   just run                      # Debug run
#   just run release              # Release run
#   just build release            # Release build
#   just release v1.2.3           # Release + tag
# =============================================================================

default:
    just --list --unsorted

# =============================================================================
# Release
# =============================================================================

release version:
    ./scripts/release.sh {{version}}

# =============================================================================
# Build & Run
# =============================================================================

build profile="debug":
    cargo build {{ if profile == "release" { "--release" } else { "" } }}

run profile="debug":
    cargo run {{ if profile == "release" { "--release" } else { "" } }}

# Target builds
build-target target profile="release":
    cargo build --target {{target}} {{ if profile == "release" { "--release" } else { "" } }}

# =============================================================================
# Dev Tools
# =============================================================================

[group("dev")]
test:
    cargo test

[group("dev")]
clean:
    cargo clean

[group("dev")]
fmt:
    cargo fmt

[group("dev")]
lint:
    cargo clippy --all-targets --all-features

[group("dev")]
check:
    cargo check

[group("dev")]
update:
    cargo update

# =============================================================================
# Utils
# =============================================================================

info:
    @echo "Project: fos_client"
    @echo "Rust: $(rustc --version)"
    @echo "Cargo: $(cargo --version)"

validate:
    @echo "✅ Setup OK" &&
    cargo --version &&
    test -f Cargo.toml

# API docs
[group("dev")]
api target="bin":
    cargo modules structure --{{target}} client
