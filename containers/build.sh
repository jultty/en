#!/usr/bin/env sh

set -eu
suffix=$(printf '%s' "$1" | sed 's/.*\.//')
tag="en:$suffix"

if podman container exists "$tag"; then
    podman stop --time 3 "$tag"
fi

if [ "$suffix" = 'debian-dev' ]; then
    cp -v ../target/x86_64-unknown-linux-gnu/debug/en en
elif [ "$suffix" = 'alpine-dev' ]; then
    cp -v ../target/x86_64-unknown-linux-musl/debug/en en
fi

podman build \
    --tag "$tag" \
    -f "Containerfile.$suffix"

if [ -f en ]; then
    rm -v en
fi
