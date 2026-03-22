#!/usr/bin/env bash
set -e

echo "=== Campfire Setup ==="

# Rust / Cargo
if ! command -v cargo &>/dev/null; then
    echo "→ Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "✓ Rust $(rustc --version)"
fi

# just
if ! command -v just &>/dev/null; then
    echo "→ Installing just..."
    cargo install just
else
    echo "✓ just $(just --version)"
fi

# Linux system dependencies (Bevy)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if command -v apt &>/dev/null; then
        echo "→ Installing system dependencies (apt)..."
        sudo apt-get update
        sudo apt-get install -y \
            libasound2-dev \
            libudev-dev \
            libx11-dev \
            libxcursor-dev \
            libxrandr-dev \
            libxi-dev \
            libvulkan-dev \
            libwayland-dev \
            libxkbcommon-dev \
            pkg-config
    else
        echo "⚠ Kein apt gefunden — bitte manuell installieren:"
        echo "  libwayland-dev libxkbcommon-dev libudev-dev libx11-dev"
        echo "  libxrandr-dev libxi-dev libgl1-mesa-dev pkg-config"
    fi
fi

echo ""
echo "✓ Setup abgeschlossen. Starten mit: just run"
