## Known issues

As documented by the source files and hashes in this directory, significant effort was made to preserve the original license files, determine that equally-licensed files have identical licenses and that the vendored license files match the upstream well-known official versions.

In particular, the vendored file for the Reforma font is in Markdown format and as such hard to match against the license file supplied by Creative Commons without substantial reductions in punctuation, spacing and external URLs.

However, because only this font uses such license, it is still a single copy that must end up embedded in the binary. As such, despite a missing matching hash, the vendored version is embedded as-is.

Due to this issue, it was decided that any future font that is a candidate for addition to the project MUST be licensed under some version of the SIL Open Font License.
