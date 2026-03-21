#!/usr/bin/env sh

set -eu

JUST_VERSION="1.47.1"
JUST_SHA256SUM="3cb931ae25860f261ee373f32ede3b772ac91f14f588e4071576d3ffcf1a16fd"
CARGO_LLVM_COV_VERSION="0.6.21"
CARGO_LLVM_COV_SHA256SUM="57f491aedf7cdb261538ceb49cbb1ee9d27df7ca205a5e1a009caaf5cb911afb"
CARGO_AUDIT_VERSION="0.22.1"
CARGO_AUDIT_TAG="cargo-audit%2Fv$CARGO_AUDIT_VERSION"
CARGO_AUDIT_SHA256SUM="1890badd5f15831a9af4b074399fcd21e6f7c0fe42c84e9254cdffc9f813765c"

TRIPLE="x86_64-unknown-linux-gnu"
TRIPLE_MUSL="x86_64-unknown-linux-musl"

fetch() {
    repo="$1"; tag="$2"; filename="$3"; digest="$4"; binary="$5"

    [ -d /tmp/tools ] || mkdir -p /tmp/tools

    curl -fsSLO --output-dir /tmp \
        -w '%{stderr}HTTP %{response_code} %{url}\n' \
        "https://github.com/$repo/releases/download/$tag/$filename"

    printf '%s  %s\n' "$digest" "/tmp/$filename" > /tmp/digest
    sha256sum --check /tmp/digest
    tar xf "/tmp/$filename" -C /tmp/tools
    find /tmp/tools -type f -executable -name "$binary" \
        -exec mv -v '{}' /usr/local/bin ';'
}

fetch casey/just "$JUST_VERSION" \
    "just-$JUST_VERSION-$TRIPLE_MUSL.tar.gz" \
    "$JUST_SHA256SUM" just

fetch taiki-e/cargo-llvm-cov "v$CARGO_LLVM_COV_VERSION" \
    "cargo-llvm-cov-$TRIPLE.tar.gz" \
    "$CARGO_LLVM_COV_SHA256SUM" cargo-llvm-cov

fetch rustsec/rustsec "$CARGO_AUDIT_TAG" \
    "cargo-audit-$TRIPLE-v$CARGO_AUDIT_VERSION.tgz" \
    "$CARGO_AUDIT_SHA256SUM" cargo-audit
