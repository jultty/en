#!/usr/bin/env sh

set -eu
distro="$1"

podman stop --time 3 "en-$distro"
podman build \
    --tag "en-$distro" \
    -f "Containerfile.$distro"
