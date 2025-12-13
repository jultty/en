_default:
    @just --list

# DEV

# Build on changes
[group('dev')]
serve-watch:
    bacon --job run-long

alias sw := serve-watch
alias dev := serve-watch
alias d := serve-watch

# Run tests on changes
[group('dev')]
test-watch:
    bacon --job test

alias tw := test-watch

# Format check on changes
[group('dev')]
format-watch:
    bacon --job fmt-check

alias fw := format-watch

# Check before push
[group('dev')]
push: check
    git push


# RUN

# Start server
[group('run')]
serve:
    cargo run

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
check: lint format-check test

alias c := check

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
test: build
    cargo test

alias t := test

# FORMATTING

# Format all files
[group('checks')]
format:
    cargo fmt

alias f := format
