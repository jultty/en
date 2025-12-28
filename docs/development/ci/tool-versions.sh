#!/usr/bin/env sh

tools="
casey/just just-%VERSION%-x86_64-unknown-linux-musl.tar.gz
taiki-e/cargo-llvm-cov cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz
"

get_release() {
   curl -sSL \
      -H "Accept: application/json" \
      "https://api.github.com/repos/$1/releases/latest"
}

q() { printf '%s' "$1" | jq -r "$2"; }

git_root=$(git rev-parse --show-toplevel)

printf '%s' "$tools" | while read -r repo asset_template; do
    [ -n "$repo" ] || continue

    release=$(get_release "$repo")
    workflow_var=$(echo "$repo" |
        awk -F / '{ gsub(/-/, "_"); print toupper($2) }')_VERSION

    latest=$(q "$release" .tag_name | tr -d v)
    current=$(grep -m 1 "$workflow_var" \
        "$git_root/.forgejo/workflows/check.yaml" | awk '{print $2}')

    echo "$repo"
    echo "In use: $current"
    echo "Latest: $latest"
    if [ "$current" != "$latest" ]; then
        echo "  Published: $(q "$release" .published_at)"
        echo "  [ Prerelease: $(q "$release" .prerelease) ]"
        echo "  URL: $(q "$release" .html_url)"
        echo "  $(q "$release" .body)"
        asset_pattern=$(printf '%s' "$asset_template" |
            sed "s/%VERSION%/$latest/g")
        asset=$(q "$release" ".assets[] | select(.name == \"$asset_pattern\")")
        if [ -n "$asset" ]; then
            echo "    Asset: $(q "$asset" .name)"
            echo "    sha256sum: $(q "$asset" .digest)"
            echo "    URL: $(q "$asset" .browser_download_url)"
        else
            echo " No asset matching pattern $asset_pattern in this release"
        fi
    fi
    echo
done
