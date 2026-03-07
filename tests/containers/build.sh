#!/usr/bin/env sh

set -eu
suffix=$(printf '%s' "$1" | sed 's/.*\.//')
tag="en:$suffix"

if podman container exists "$tag"; then
    podman stop --time 3 "$tag"
fi

if [ "$suffix" = 'debian-dev' ]; then
    cp ../../target/release/en en
elif [ "$suffix" = 'alpine-dev' ]; then
    cp ../../target/x86_64-unknown-linux-musl/release/en en
fi

podman build \
    --tag "$tag" \
    -f "Containerfile.$suffix"

rm en
