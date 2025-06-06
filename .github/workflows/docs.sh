#!/usr/bin/env -S nix develop .#github-ci --command bash

# Clean docs folder
cargo clean --doc

# Build docs
cargo doc --no-deps --document-private-items --examples --lib
