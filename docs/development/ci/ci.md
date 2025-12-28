# Continuous Integration

A workflow defined at `.forgejo/workflows/check.yaml` performs several checks
as defined in `.justfile`. This allows for the same commands to be run locally
to similar effect.

Beware, however, some differences in the CI environment:

- Git doesn't track empty directories, so any of those will be absent
- The current image, [rust:slim][1], is based on Debian stable

[1]: https://github.com/rust-lang/docker-rust/blob/c2c1f65/stable/trixie/slim/Dockerfile

## Updating binaries fetched in CI

The CI workflow has some hardcoded versions for the following tooling:

| Tool               | Repository                  | Asset filename pattern                           |
|:------------------:|:---------------------------:|:------------------------------------------------:|
| **just**           | [casey/just][2]             | `just-1.45.0-x86_64-unknown-linux-musl.tar.gz`   |
| **cargo-llvm-cov** | [taiki-e/cargo-llvm-cov][3] | `cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz` |

[2]: https://github.com/casey/just
[3]: https://github.com/taiki-e/cargo-llvm-cov

These should be relatively recent to keep up with development environments, but
don't really need to change often unless an interesting feature or security
patch is available. When updates are desirable, they must be done manually so
the actual changes behind each release can be properly reviewed and tested.

## Determining latest release

The `tool-versions.sh` script available in this directory checks for the last releases
from the tooling currently used. If a new release is available, it prints out some
relevant information to decide about doing a CI workflow update.

To update the version, just edit the `env` section at the top of the workflow
file with the new version tag and corresponding sha256sum value. Note this is a hash
output for the tarball, not the binary. The script can fetch the correct digest.

### Getting tooling release data from the GitHub API

The `GET` endpoint `/repos/{owner}/{repo}/releases` can be used to list the
existing releases.

```sh
curl -L \
    -H "Accept: application/json" \
    https://api.github.com/repos/OWNER/REPO/releases
```

This will return an array of release objects starting from the latest releases.

Relevant response fields in these objects include:

- `.tag_name`: Tag for specific, more detailed queries including assets (see
below)
- `.prerelease`: Release was marked as prerelease. If true, don't use the
release in CI.
- `.html_url`: Sometimes includes notes with notable release changes.
- `.body`: Same as above, but directly in the response body.
- `.created_at`: Date the release was created (even if as a draft)
- `.published_at`: Date the release was published
published)

The `GET`endpoints `/repos/{owner}/{repo}/releases/latest` and
`/repos/{owner}/{repo}/releases/tags/{tag}` can be used to get information on
the very latest release and on a specific release as referenced by its tag.

```sh
curl -L \
    -H "Accept: application/json" \
    https://api.github.com/repos/OWNER/REPO/releases/latest
```

```sh
curl -L \
    -H "Accept: application/json" \
    https://api.github.com/repos/OWNER/REPO/releases/tags/TAG
```

For both of these endpoints, relevant fields include:

- `.tag_name`: From the `latest` endpoint, allows constructing a canonical URI
for the release
- `.html_url`, `.body`, `.prerelease`: See above

- `.assets`: An array of asset objects
- `.assets_url`: A URL that returns the assets array
- `.assets.[].name`: File name, to be matched with the table above
- `.assets.[].digest`: A sha256sum hash value
- `.assets.[].url`: A URL that returns a given asset object alone
- `.assets.[].browser_download_url` A download URL to fetch the actual asset

For the GitHub API reference, see
<https://docs.github.com/en/rest/releases/releases?apiVersion=2022-11-28>.

## Notes

Some other notes relevant to modifying the CI workflow:

- `$PATH` in CI is
`/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`
as of [docker-rust](https://github.com/rust-lang/docker-rust) commit `c2c1f65`,
2025-12-27
