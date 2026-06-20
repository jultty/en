#!/usr/bin/env sh

set -eu

JUST_VERSION="1.51.0"
JUST_SHA256SUM="c8f085ca3e885723c341d06243fc291b5abfdc8bbe3b2c076b117de490387b59"
CARGO_LLVM_COV_VERSION="0.6.21"
CARGO_LLVM_COV_SHA256SUM="57f491aedf7cdb261538ceb49cbb1ee9d27df7ca205a5e1a009caaf5cb911afb"
CARGO_AUDIT_VERSION="0.22.1"
CARGO_AUDIT_TAG="cargo-audit%2Fv$CARGO_AUDIT_VERSION"
CARGO_AUDIT_SHA256SUM="1890badd5f15831a9af4b074399fcd21e6f7c0fe42c84e9254cdffc9f813765c"
TAPLO_VERSION="0.10.0"
TAPLO_SHA256SUM="8fe196b894ccf9072f98d4e1013a180306e17d244830b03986ee5e8eabeb6156"
TYPOS_VERSION="1.47.2"
TYPOS_SHA256SUM="7aef58932fc123b4cf4b40d86468e89a3297d80169051d7cfd13a235e05fc426"
CARGO_DENY_VERSION="0.19.8"
CARGO_DENY_SHA256SUM="70e769ae3872e34d45132b17040859175e11401dc12dddb0303e0b8c7d088f3f"
CARGO_XWIN_VERSION="0.23.0"
CARGO_XWIN_SHA256SUM="74a216f64f10ea81c909f02d6b1a84cd0fda8de4c87ee52fe63ba76ab2392b75"

TRIPLE="x86_64-unknown-linux-gnu"
TRIPLE_MUSL="x86_64-unknown-linux-musl"

fetch() {
    repo="$1"
    tag="$2"
    filename="$3"
    digest="$4"
    binary="$5"
    renamed_binary="${6:-$binary}"

    [ -d /tmp/tools ] || mkdir -p /tmp/tools

    curl -fsSLO --output-dir /tmp \
        -w '%{stderr}HTTP %{response_code} %{url}\n' \
        "https://github.com/$repo/releases/download/$tag/$filename"

    printf '%s  %s\n' "$digest" "/tmp/$filename" > /tmp/digest
    sha256sum --check /tmp/digest

    if printf '%s' "$filename" | grep -qE '\.tar\.|\.tgz$'; then
        tar xf "/tmp/$filename" -C /tmp/tools
    elif printf '%s' "$filename" | grep -q '\.gz$'; then
        gunzip -c "/tmp/$filename" > "/tmp/tools/$binary"
    else
        echo "Fatal: can't determine how to unpack $filename"
        exit 1
    fi

    find /tmp/tools -type f -name "$binary" \
        -exec mv -v '{}' "/usr/local/bin/$renamed_binary" ';'
    chmod +x "/usr/local/bin/$renamed_binary"
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

fetch tamasfe/taplo "$TAPLO_VERSION" \
    "taplo-linux-x86_64.gz" \
    "$TAPLO_SHA256SUM" taplo-linux-x86_64 taplo

fetch crate-ci/typos "v$TYPOS_VERSION" \
    "typos-v$TYPOS_VERSION-$TRIPLE_MUSL.tar.gz" \
    "$TYPOS_SHA256SUM" typos

fetch EmbarkStudios/cargo-deny "$CARGO_DENY_VERSION" \
    cargo-deny-$CARGO_DENY_VERSION-$TRIPLE_MUSL.tar.gz \
    "$CARGO_DENY_SHA256SUM" cargo-deny

fetch rust-cross/cargo-xwin "v$CARGO_XWIN_VERSION" \
    cargo-xwin-v$CARGO_XWIN_VERSION.$TRIPLE_MUSL.tar.gz \
    "$CARGO_XWIN_SHA256SUM" cargo-xwin
