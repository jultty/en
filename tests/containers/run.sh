#!/usr/bin/env sh

set -eu

distro="$1"

podman run --replace --name "en-$distro" --publish 3008:3008 "en-$distro"
