#!/usr/bin/env sh

set -eu

get_marker() {
    # extended regex syntax
    cc_marker='^\#? ?Attribution-NoDerivatives 4.0 International$'
    ofl_marker='^SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007$'

    m_file="$1"

    if grep -Eq "$ofl_marker" "$m_file"; then
        printf '%s' "$ofl_marker"
    elif grep -Eq "$cc_marker" "$m_file"; then
        printf '%s' "$cc_marker"
    else
        printf '%s %s\n' "$m_file" "matches no marker" >&2
        printf ''
    fi
}

behead() {
    b_file="$1"
    b_marker="$2"
    b_out_path="$3"

    sed -E "/$b_marker/, \$d" "$b_file" | sed -E 's/^-+$//' > "$b_out_path"
}

compact() {
    c_file="$1"
    c_marker="$2"
    c_out_path="$3"

    # Eliminating [:space:] is enough for OFL to match, but Reforma's
    # markdown license file is full of quirks so we must reduce aggresively
    sed -En "/$c_marker/, \$p" "$c_file" \
        | sed -E 's/(wiki\.creativecommons\.org|https?).*\s//g' \
        | tr -cd '[:alnum:]' \
        | tr '[:upper:]' '[:lower:]' > "$c_out_path"
}

for dir in *; do
    [ -d "$dir" ] || continue
    [ "$dir" != _licenses ] || continue
    license="$dir/LICENSE"; [ -f "$license" ]

    marker=$(get_marker "$license")

    if [ -n "$marker" ]; then
        behead "$license" "$marker" "$dir/header.LICENSE"
        compact "$license" "$marker" "$dir/compact.LICENSE"
    fi
done

for license in _licenses/*.LICENSE; do
    printf '%s' "$license" | grep -qEv '(header|compact)\.LICENSE' || continue

    marker=$(get_marker "$license")
    compact_name=$(printf '%s' "$license" | sed "s*\.LICENSE*.compact.LICENSE*")
    compact "$license" "$marker" "$compact_name"
done

for file in ./*/*LICENSE*; do
    size=$(du "$file" | awk '{print $1}')
    if [ "$size" -le 0 ]; then
        echo "$file is empty"
        exit 1
    fi
done

sha256sum ./*/*LICENSE* > LICENSES.sha256sum
grep compact LICENSES.sha256sum | sort

unique_licenses=$(
    find _licenses/ -name '*.LICENSE' -not -name '*.compact.LICENSE' | wc -l
)
unique_hashes=$(
    cat LICENSES.sha256sum | grep compact | awk '{print $1}' | sort | uniq | wc -l
)

if [ "$unique_hashes" -ne "$unique_licenses" ]; then
    echo "unique hashes: $unique_hashes"
    echo "unique licenses: $unique_licenses"
    exit 1
fi

