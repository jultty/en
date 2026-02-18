# DEVELOP

# Update dependencies
[group: 'develop']
update:
    cargo update --verbose

# Build and serve
[group: 'develop']
run host='::1' port='3003' *args:
    DEBUG=${DEBUG:-debug} {{ debug_vars }} cargo run -- \
        --hostname {{ host }} --port {{ port }} {{ args }}

alias r := run

# Build and serve on changes
[group: 'develop']
run-watch:
    @{{ watch_cmd }} {{ just_cmd }} run

alias w := run-watch

[private]
quick-assess:
    {{ just_cmd }} lint check quick-test-cover

[private]
quick-assess-run:
    -{{ just_cmd }} quick-assess
    {{ just_cmd }} run

# Run quick assessments on changes
[group: 'develop']
quick-assess-watch:
    {{ watch_cmd }} {{ just_cmd }} quick-assess

alias qa := quick-assess-watch

# Run quick assessments, build and serve on changes
[group: 'develop']
quick-assess-run-watch:
    {{ watch_cmd }} {{ just_cmd }} quick-assess-run

alias qr := quick-assess-run-watch

[private]
quick-test-cover:
    {{ cover_cmd }} --no-report -- --test 'serial_tests::' --test-threads 1
    {{ cover_cmd }} --no-report -- --skip 'serial_tests::'
    {{ cover_cmd }} report --html
    @{{ cover_cmd }} report | tail -1 | awk '{ print " [ Regions:", $4, "• Functions:", $7, "• Lines:", $10, "]" }'

# Quickly update coverage reports (inaccurate)
[group: 'assess']
quick-test-cover-watch:
    {{ watch_cmd }} {{ just_cmd }} quick-test-cover

alias qo := quick-test-cover-watch

# Format all files
[group: 'develop']
format:
    cargo +nightly fmt

alias f := format

# Lint
[group: 'develop']
lint:
    cargo +nightly clippy

alias l := lint

# Lint on changes
[group: 'develop']
lint-watch:
    {{ watch_cmd }} {{ just_cmd }} lint

alias lw := lint-watch

# Run cargo check on changes
[group: 'develop']
check-watch:
    RUSTFLAGS="-Dwarnings" {{ watch_cmd }} {{ just_cmd }} check

alias cw := check-watch

# Apply rustc lint fixes
[group: 'develop']
rustc-fix:
    cargo fix --allow-dirty

alias rf := rustc-fix

# Apply clippy lint fixes
[group: 'develop']
clippy-fix:
    cargo +nightly clippy --fix --allow-dirty

alias cf := clippy-fix

# Apply all automatic fixes
[group: 'develop']
fix: rustc-fix clippy-fix format

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
cover-open:
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
    cargo +nightly fmt -- --check

alias fc := format-assess

# Assess production lints
[group: 'assess']
lint-assess:
    cargo +nightly clippy -- \
        -D clippy::dbg_macro -D clippy::use_debug \
        -D clippy::print_stdout -D clippy::print_stderr \
        -D clippy::todo -D clippy::unimplemented -D clippy::unreachable

alias la := lint-assess

# Run cargo check
[group: 'assess']
check:
    {{ debug_vars }} cargo check --workspace

alias c := check

# Run tests
[group: 'assess']
test:
    cargo test -- --test 'serial_tests::' --test-threads 1
    cargo test --bin en
    cargo test --doc
    cargo test --lib -- --skip 'serial_tests::'

alias t := test

# Clean test coverage data
[group: 'assess']
test-cover-clean:
    {{ cover_cmd }} clean

alias oc := test-cover-clean

# Run tests with coverage
[group: 'assess']
test-cover: test-cover-clean
    {{ cover_cmd }} --no-report -- --test 'serial_tests::' --test-threads 1
    {{ cover_cmd }} --no-report -- --skip 'serial_tests::'

alias o := test-cover

# Assess coverage
[group: 'assess']
cover-assess: test-cover
    {{ cover_cmd }} --fail-under-regions 95 report

alias oa := cover-assess

# Run all assessments
[script, group: 'assess']
verify:
    export RUSTFLAGS=${RUSTFLAGS:-"-Dwarnings"}
    if [ -n "$(git status --porcelain)" ]; then
        echo "Git working tree is dirty: Commit or stash your changes first"
        git status
        exit 1
    fi
    {{ just_cmd }} update version-assess format-assess lint-assess check test cover-assess

alias v := verify

# Check tag-manifest consistency
[script, group: 'assess']
version-assess:
    last_tag=$(git describe --tags --abbrev=0 \
        $(git rev-list --tags --max-count=1) | tr -d v)
    manifest_version=$(grep '^version' Cargo.toml | cut -d \" -f 2)
    lockfile_version=$(grep -A 1 'name = "en"' Cargo.lock |
        grep version | cut -d '"' -f 2)
    [ "$last_tag" = "$manifest_version" ]
    [ "$last_tag" = "$lockfile_version" ]

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
full-build: clean release-build

alias fb := full-build

# Upload release build to git.jutty.dev package registry
[script, group: 'build']
upload: full-build && shasum
    version=$(./target/release/en --version)
    api_root=https://git.jutty.dev/api/
    url=$api_root/packages/jutty/generic/en/$version/en-x86_64-linux-gnu
    file=target/release/en

    curl -fsSL \
        --user jutty:$(secret-tool lookup Title gjd-registry-token) \
        --upload-file $file $url

alias u := upload

# Print sha256sum for CI logging
[group: 'build']
shasum:
    sha256sum target/release/en

## META

[default, private]
default:
    @just --list --unsorted --justfile {{justfile()}}

export CARGO_TERM_COLOR := 'always'

debug_vars := 'DEBUG=${DEBUG:-} DEBUG_FILTER=${DEBUG_FILTER:-} RUST_BACKTRACE=${RUST_BACKTRACE:-} RUSTFLAGS=${RUSTFLAGS:-}'
watch_cmd := "watchexec -qc -r -e rs,toml,html --color always -- "
cover_cmd := 'cargo llvm-cov --color always --ignore-filename-regex "main\.rs|log\.rs"'
just_cmd := 'just --timestamp --explain --command-color green'

set unstable
