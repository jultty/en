# Notes

## BTreeMap

Consider replacing HashMap with BTreeMap to stop nodes from shifting position constantly on every page load.

See also:
    - <https://doc.rust-lang.org/std/collections/struct.BTreeMap.html#background>
    - `clippy::iter_over_hash_type`


## Overall guidelines

- Take refs, return owned
- Avoid opacity
    - Third-party macros
    - Procedural macro attributes
    - Returning opaque types, like `impl Trait`
