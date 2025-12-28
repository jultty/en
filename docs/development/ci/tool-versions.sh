#!/usr/bin/env sh

tools="
casey/just just-%VERSION%-x86_64-unknown-linux-musl.tar.gz
taiki-e/cargo-llvm-cov cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz
"

git_root=$(git rev-parse --show-toplevel)

get_release() {
    repo="$1"

   curl -sSL \
      -H "Accept: application/json" \
      "https://api.github.com/repos/$repo/releases/latest"
}

q() { printf '%s' "$1" | jq -r "$2"; }

printf '%s' "$tools" | while read -r repo asset_template; do
    [ -n "$repo" ] || continue
    release=$(get_release "$repo")
    workflow_var=$(echo "$repo" | tr '[:lower:]' '[:upper:]' | tr '-' '_' | cut -d '/' -f 2)_VERSION
    current=$(grep -m 1 "$workflow_var" "$git_root/.forgejo/workflows/check.yaml" | awk '{print $2}')
    latest=$(q "$release" .tag_name | tr -d v)
    echo "$repo"
    echo "In use: $current"
    echo "Latest: $latest"
    if [ "$current" != "$latest" ]; then
        echo "  Published: $(q "$release" .published_at)"
        echo "  [ Draft: $(q "$release" .draft) ] [ Prerelease: $(q "$release" .prerelease) ]"
        echo "  URL: $(q "$release" .html_url)"
        echo "  $(q "$release" .body)"
        asset_pattern=$(printf '%s' "$asset_template" | sed "s/%VERSION%/$latest/g")
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
