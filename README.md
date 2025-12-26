# en

en is a tool to write non-linear, connected pieces of text and have their references mapped out as a graph of connected information.

It works by ingesting a TOML file containing your node specification and serving it as a website that allows nodes to be browsed, searched and listed in relation to each other or as a shallow tree of nodes.

## Learn more

You can learn more and see what en looks like by visiting the [homepage](https://en.jutty.dev), which is rendered using en itself.

## Roadmap

- [x] Add tests
    - [ ] Improve content syntax parser coverage
- [ ] Richer text formatting
    - [x] Headers
    - [x] Preformatted blocks
    - [x] Inline code
    - [x] Anchor rendering
        - [ ] Automatic anchors
        - [ ] `#` syntax for header ID anchors
    - [x] External anchors
    - [ ] Bold, italics, underline, strikethrough
    - [ ] Lists
    - [ ] Checkboxes
        - [ ] Move this roadmap to en
- [ ] Connection kinds
    - [ ] Mutual
    - [ ] Category <-> Membership
    - [ ] Opposite <-> Equivalent
    - [ ] Contrast <-> Similar
    - [ ] Cognate <-> Unrelated
    - [ ] Specialization <-> Generalization
    - [ ] Custom connection kinds
- [ ] Strip/render some syntax in Tree text preview
- [ ] Begin centralizing state
- [ ] Full-text search
- [ ] Render to filesystem
- [ ] Reduce O(n) calls in the formats module
- [ ] Multi-file graphs
- [ ] Multi-graph
- [ ] Themes
- [x] Array syntax for lightweight connections
- [x] Automatic IDs
- [x] Automatic titles
- [x] Mismatch between TOML ID and provided ID
