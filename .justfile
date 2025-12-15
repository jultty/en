_default:
    @just --list

# DEV

# Build on changes
[group('dev')]
serve-watch:
    bacon --job run-long -- -- --hostname localhost --port 3003

alias sw := serve-watch
alias dev := serve-watch
alias d := serve-watch

# Run tests on changes
[group('dev')]
test-watch:
    bacon --job test

alias tw := test-watch

# Run cargo check on changes
[group('dev')]
check-watch:
    bacon --job check

alias cw := check-watch

# Format check on changes
[group('dev')]
format-watch:
    bacon --job fmt-check

alias fw := format-watch

# Lint on changes
[group('dev')]
lint-watch:
    bacon --job clippy

alias lw := lint-watch

# Check before push
[group('dev')]
push: check
    git push


# RUN

# Start server
[group('run')]
serve:
    cargo run -- --hostname localhost --port 3003

alias s := serve

# BUILD

# Build project with Cargo
[group('build')]
build:
    cargo build

alias b := build

# Cleanup build artifacts
[group('build')]
clean:
    cargo clean

alias cl := clean

# Clean, build, run checks
[group('build')]
full-build: clean build check

alias fb := full-build

# Release build
[group('build')]
release-build:
    cargo build --release

alias rb := release-build

# CHECKS

# Lint, check formatting and run tests
[group('checks')]
check: format-check lint cargo-check test

alias c := check

# Run cargo check
[group('checks')]
cargo-check:
    cargo check --workspace

alias cc := cargo-check

# Lint with Clippy
[group('checks')]
lint:
    cargo clippy

alias l := lint

# Check formatting without changing files
[group('checks')]
format-check:
    cargo fmt -- --check

alias fc := format-check

# Run tests
[group('checks')]
test:
    cargo test

alias t := test

# FORMATTING

# Format all files
[group('checks')]
format:
    cargo fmt

alias f := format
