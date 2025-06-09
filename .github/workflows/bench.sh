#!/usr/bin/env -S nix develop .#github-ci --command bash
# shellcheck disable=SC1008

# Build benchmarks
cargo bench
