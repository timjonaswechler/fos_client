# =============================================================================
# Justfile: Build Automation für fos_client
# =============================================================================
# Dieses Justfile bietet konsistente Build-Befehle für lokale Entwicklung.
# Die CI/CD Pipeline verwendet GitHub Actions (siehe .github/workflows/)
#
# Usage:
#   just run              # Debug-Build und Ausführung
#   just run release      # Release-Build und Ausführung
#   just build release    # Nur Release-Build
#   just release 1.0.0    # Neues Release erstellen und pushen
# =============================================================================
# Standard Steam App ID für Entwicklung (SpaceWar)

steam_app_id := "{{env_var('STEAM_APP_ID')}}"

# =============================================================================
# Release Management
# =============================================================================

# Erstellt ein neues Release (Version bump + Tag + Push)
release version:
    ./scripts/release.sh {{ version }}

# =============================================================================
# Build Commands
# =============================================================================

# Build das Projekt (debug oder release)
build profile="debug":
    @if [ "{{ profile }}" = "release" ]; then \
      echo "Building RELEASE with Steam App ID {{ steam_app_id }}..."; \
      STEAM_APP_ID={{ steam_app_id }} cargo build --release; \
    else \
      echo "Building DEBUG with Steam App ID {{ steam_app_id }}..."; \
      STEAM_APP_ID={{ steam_app_id }} cargo build; \
    fi

# Build für spezifisches Target (Cross-Compilation)
build-target target profile="release":
    echo "Building for target: {{ target }} ({{ profile }})"
    STEAM_APP_ID={{ steam_app_id }} cargo build --target {{ target }} {{ if profile == "release" { "--release" } else { "" } }}

# =============================================================================
# Run Commands
# =============================================================================

# Build und führe das Projekt aus
run profile="debug":
    @if [ "{{ profile }}" = "release" ]; then \
      echo "Running RELEASE build with Steam App ID {{ steam_app_id }}..."; \
      STEAM_APP_ID={{ steam_app_id }} cargo run --release; \
    else \
      echo "Running DEBUG build with Steam App ID {{ steam_app_id }}..."; \
      STEAM_APP_ID={{ steam_app_id }} cargo run; \
    fi

# =============================================================================
# Development Commands
# =============================================================================

[group('dev')]
test:
    STEAM_APP_ID={{ steam_app_id }} cargo test

[group('dev')]
clean:
    cargo clean

[group('dev')]
fmt:
    cargo fmt

[group('dev')]
lint:
    cargo clippy --all-targets --all-features

[group('dev')]
check:
    cargo check

# =============================================================================
# Utility Commands
# =============================================================================

# Zeigt Informationen zum Projekt
info:
    @echo "Project: fos_client"
    @echo "Steam App ID: {{ steam_app_id }}"
    @cargo --version
    @rustc --version

# Validiert das Setup (prüft Dependencies)
validate:
    @echo "Validating project setup..."
    @cargo --version || (echo "ERROR: Rust/Cargo not found" && exit 1)
    @test -f Cargo.toml || (echo "ERROR: Cargo.toml not found" && exit 1)
    @echo "Setup validation passed!"

# Erstellt steam_appid.txt für lokale Entwicklung
setup-dev:
    @echo "Creating steam_appid.txt for development..."
    @echo "{{ steam_app_id }}" > steam_appid.txt
    @echo "Created steam_appid.txt with App ID {{ steam_app_id }}"
    @echo "Note: steam_appid.txt is in .gitignore and won't be committed"

# =============================================================================
# API/Structure Analysis
# =============================================================================

[group('dev')]
api type="bin":
    @if [ "{{ type }}" = "bin" ]; then \
      just api-bin; \
    else \
      just api-other "{{ type }}"; \
    fi

_api-bin:
    cargo modules structure --bin client

_api-other type:
    cargo modules structure --{{ type }}
