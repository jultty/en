_default:
    @just --list

watch_cmd := "watchexec -qc -r -e rs,toml,html --color always -- "
cover_cmd := 'cargo llvm-cov --ignore-filename-regex "main\.rs|dev\.rs"'
just_cmd := 'just --unstable --timestamp --explain --command-color green'

# DEV

# Start server
[group: 'develop']
run:
    cargo run -- --hostname localhost --port 3003

alias r := run

# Build on changes
[group: 'develop']
run-watch:
    {{ watch_cmd }} {{ just_cmd }} run

alias rw := run-watch
alias dev := run-watch
alias d := run-watch

# Run all assessments on changes
[group: 'develop']
verify-watch:
    {{ watch_cmd }} {{ just_cmd }} verify

alias vw := verify-watch

# Run tests on changes
[group: 'develop']
test-watch:
    {{ watch_cmd }} {{ just_cmd }} test

alias tw := test-watch

# Run tests with coverage reports on changes
[group: 'develop']
cover-watch:
    {{ watch_cmd }} {{ just_cmd }} cover-report

alias ow := cover-watch

# Run cargo check on changes
[group: 'develop']
check-watch:
    {{ watch_cmd }} {{ just_cmd }} check

alias cw := check-watch

# Lint on changes
[group: 'develop']
lint-watch:
    {{ watch_cmd }} {{ just_cmd }} lint

alias lw := lint-watch

# Assess formatting on changes
[group: 'develop']
format-watch:
    {{ watch_cmd }} {{ just_cmd }} format-assess

alias fw := format-watch

# Format all files
[group: 'develop']
format:
    cargo fmt

alias f := format

# Verify before push
[group: 'develop']
push: verify
    git push

alias p := push

# ANALYSIS

# Run all analysis
[group: 'assess']
verify: format-assess lint check test cover-assess

alias v := verify

# Assess coverage
[group: 'assess']
cover-assess:
    {{ cover_cmd }} --fail-under-regions 90 report

# Assess formatting
[group: 'assess']
format-assess:
    cargo fmt -- --check

alias fc := format-assess

# Lint with Clippy
[group: 'assess']
lint:
    cargo clippy

alias l := lint

# Run cargo check
[group: 'assess']
check:
    cargo check --workspace

alias c := check

# Run tests
[group: 'assess']
test:
    cargo test -- --skip 'serial_tests::'
    cargo test -- --test 'serial_tests::' --test-threads 1

alias t := test

# Run tests with coverage
[group: 'assess']
cover:
    {{ cover_cmd }} --no-report -- --skip 'serial_tests::'
    {{ cover_cmd }} --no-report -- --test 'serial_tests::' --test-threads 1

alias o := cover

## COVER

# Make coverage report
[group: 'cover']
cover-report: cover
    {{ cover_cmd }} report --html
    {{ cover_cmd }} report

alias or := cover-report

# Open coverage report
[group: 'cover']
cover-open: cover
    {{ cover_cmd }} report --open

alias oo := cover-open

# BUILD

# Build project with Cargo
[group: 'build']
build:
    cargo build

alias b := build

# Cleanup build artifacts
[group: 'build']
clean:
    cargo clean

alias cl := clean

# Clean, run assessments, release build
[group: 'build']
full-build: clean verify release-build

alias fb := full-build

# Release build
[group: 'build']
release-build: verify
    cargo build --release

alias rb := release-build
