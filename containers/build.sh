#!/usr/bin/env sh

set -eu
suffix=$(printf '%s' "$1" | sed 's/.*\.//')
tag="en:$suffix"
shift

if podman container exists "$tag"; then
    podman stop --time 3 "$tag"
fi

case "$suffix" in
    *-dev)
        rsync -a \
        --exclude /target \
        --exclude .git \
        --exclude /containers \
        .. dev-src
esac

podman build \
    --tag "$tag" \
    -f "Containerfile.$suffix" "$@"

if [ -d dev-src ]; then
    rm -r dev-src
fi
