# DEVELOP

# Update dependencies
[group: 'develop']
update:
    cargo update --verbose

alias u := update

# Build and run server
[group: 'develop']
run host='::1' port='3003' *args:
    {{ debug_vars }} cargo run -- \
        --hostname {{ host }} --port {{ port }} {{ args }}

alias r := run

# Build and run on changes
[group: 'develop']
run-watch:
    {{ watch_cmd }} {{ just_cmd }} run

alias w := run-watch

# Format all files
[group: 'develop']
format:
    cargo fmt

alias f := format

# Lint
[group: 'develop']
lint:
    cargo clippy

alias l := lint

# Lint on changes
[group: 'develop']
lint-watch:
    {{ watch_cmd }} {{ just_cmd }} lint

alias lw := lint-watch

# Run cargo check on changes
[group: 'develop']
check-watch:
    {{ watch_cmd }} {{ just_cmd }} check

alias cw := check-watch

# Apply rustc lint fixes
[group: 'develop']
rustc-fix:
    cargo fix --allow-dirty

alias rf := rustc-fix

# Apply all automatic fixes
[group: 'develop']
fix: rustc-fix format

alias x := fix

# Run tests on changes
[group: 'develop']
test-watch:
    {{ watch_cmd }} {{ just_cmd }} test

alias tw := test-watch

# Run tests with coverage report on changes
[group: 'develop']
cover-watch:
    {{ watch_cmd }} {{ just_cmd }} cover-report

alias ow := cover-watch

# Make coverage report
[group: 'develop']
cover-report: test-cover
    {{ cover_cmd }} report --html
    {{ cover_cmd }} report

alias or := cover-report

# Open coverage report
[group: 'develop']
cover-open: test-cover
    {{ cover_cmd }} report --open

alias oo := cover-open

# Verify and push
[group: 'develop']
push: verify
    git push

alias p := push

# Generate crate documentation
[group: 'document']
doc:
    cargo doc --document-private-items --no-deps

alias d := doc

# Generate crate and dependencies documentation
[group: 'document']
doc-all:
    cargo doc --document-private-items

alias da := doc-all

# Open documentation
[group: 'document']
doc-open: doc
    xdg-open target/doc/en/index.html

alias do := doc-open

# ASSESSMENTS

# Assess formatting
[group: 'assess']
format-assess:
    cargo fmt -- --check

alias fc := format-assess

# Assess production lints
[group: 'assess']
lint-assess:
    cargo clippy -- \
        -D clippy::dbg_macro -D clippy::print_stdout -D clippy::print_stderr \
        -D clippy::todo -D clippy::unimplemented -D clippy::unreachable

alias la := lint-assess

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
test-cover:
    {{ cover_cmd }} --no-report -- --skip 'serial_tests::'
    {{ cover_cmd }} --no-report -- --test 'serial_tests::' --test-threads 1

alias o := test-cover

# Assess coverage
[group: 'assess']
cover-assess: test-cover
    {{ cover_cmd }} --fail-under-regions 95 report

# Run all assessments
[group: 'assess']
verify: format-assess lint-assess check test cover-assess

alias v := verify

# BUILD

# Cleanup build artifacts
[group: 'build']
clean:
    cargo clean

alias cl := clean

# Build project with Cargo
[group: 'build']
build: update
    cargo build

alias b := build

# Release build
[group: 'build']
release-build: update verify
    cargo build --release

alias rb := release-build

# Clean, run assessments, release build
[group: 'build']
full-build: clean update verify release-build

alias fb := full-build

## META

[default]
_default:
    @just --list --unsorted --justfile {{justfile()}}

export RUSTFLAGS := "-Dwarnings"
export RUST_BACKTRACE := "1"
export CARGO_TERM_COLOR := 'always'

debug_vars := 'DEBUG=${DEBUG:-} DEBUG_FILTER=${DEBUG_FILTER:-}'
watch_cmd := "watchexec -qc -r -e rs,toml,html --color always -- "
cover_cmd := 'cargo llvm-cov --color always --ignore-filename-regex "main\.rs|dev\.rs"'
just_cmd := 'just --timestamp --explain --command-color green'
