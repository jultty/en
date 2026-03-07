#!/usr/bin/env sh

set -eu
suffix=$(printf '%s' "$1" | sed 's/.*\.//')
name="en-$suffix"
tag="en:$suffix"

podman run \
    --replace \
    --name "$name" \
    --publish 3008:80 \
    "$tag"
