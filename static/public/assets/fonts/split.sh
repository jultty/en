#!/usr/bin/env sh

set -eu

log() {
    printf ' [split] %s\n' "$@" >&2
}

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
        log "$m_file matches no marker"
        printf ''
    fi
}

behead() {
    b_file="$1"
    b_marker="$2"
    b_head_out_path="$3"
    b_body_out_path="$4"

    log "Beheading $b_file on marker $b_marker"
    b_body=$(sed -En "/$b_marker/, \$p" "$b_file")
    b_head=$(sed -E "/$b_marker/, \$d" "$b_file")

    if [ -n "$b_head_out_path" ]; then
        log "Keeping head of $b_file on $b_head_out_path"
        printf '%s\n' "$b_head" \
        | sed -E 's/^-+$//' \
        > "$b_head_out_path"
    fi

    if [ -n "$b_body_out_path" ]; then
        log "Keeping body of $b_file on $b_body_out_path"
        printf '%s\n' "$b_body" > "$b_body_out_path"
    fi

}

compact() {
    c_file="$1"
    c_marker="$2"
    c_out_path="$3"

    sed -En "/$c_marker/, \$p" "$c_file" \
        | tr -d '[:space:]' \
        > "$c_out_path"
}

log "Iterating over font directories"
for dir in *; do
    [ -d "$dir" ] || continue
    [ "$dir" != _canon ] || continue
    license="$dir/LICENSE"; [ -f "$license" ]

    log "On license $license"

    marker=$(get_marker "$license")
    log "Got '$marker' marker for license $license"

    if [ -n "$marker" ]; then
        behead "$license" "$marker" "$dir/header.LICENSE" ""
        compact "$license" "$marker" "$dir/compact.LICENSE"
    fi
done

log "Iterating over canonical licenses"
for license in _canon/*.LICENSE; do
    printf '%s' "$license" \
        | grep -qEv '(header|compact|body)\.LICENSE' \
        || continue

    marker=$(get_marker "$license")
    log "Got '$marker' marker for license $license"

    body_name=$(printf '%s' "$license" | sed "s*\.LICENSE*.body.LICENSE*")
    compact_name=$(printf '%s' "$license" | sed "s*\.LICENSE*.compact.LICENSE*")
    log "Using names $body_name (body) and $compact_name (compact)"

    behead "$license" "$marker" "" "$body_name"
    compact "$license" "$marker" "$compact_name"
done

for file in ./*/*LICENSE*; do
    size=$(du "$file" | awk '{print $1}')
    if [ "$size" -le 0 ]; then
        log "$file is empty"
        exit 1
    fi
done

sha256sum ./*/*LICENSE* > LICENSES.sha256sum
grep compact LICENSES.sha256sum | sort

unique_licenses=$(
    find _canon/ \
        -name '*.LICENSE' \
        -not -name '*.compact.LICENSE' \
        -not -name '*.body.LICENSE' \
        | wc -l
)
unique_hashes=$(
    cat LICENSES.sha256sum \
        | grep compact \
        | awk '{print $1}' \
        | sort \
        | uniq \
        | wc -l
)

if [ "$unique_hashes" -ne "$unique_licenses" ]; then
    log "Number of distinct hashes and licenses don't match."
    log "unique hashes: $unique_hashes"
    log "unique licenses: $unique_licenses"
    exit 1
fi

