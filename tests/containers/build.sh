#!/usr/bin/env sh

set -eu

distro="$1"

podman build --tag "en-$distro" -f "Containerfile.$distro"
