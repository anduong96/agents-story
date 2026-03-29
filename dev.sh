#!/usr/bin/env bash
set -e

# Install cargo-watch if not present
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch..."
    cargo install cargo-watch
fi

# Run demo with hot reload
exec cargo watch -x 'run -- --demo --extreme'
