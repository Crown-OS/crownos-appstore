# crownos-appstore

The CrownOS App Store: the app, the app database it browses, and the packaging
that ships both, in one Cargo workspace.

| Path | What |
|---|---|
| `src/`, `data/` | the CrownUI app and `crownos-appstore-helper`, the only code that runs as root |
| `crates/apps-database/` | the database schema, validation and the published index format; `cargo build-index` |
| `database/` | one folder per app (`apps/<id>/app.toml` and its icon), plus maintenance scripts — see [database/README.md](database/README.md) |
| `packaging/` | `crownos-appstore` and `snapd` PKGBUILDs |

## Sources

| Source | Through | Needs a password |
|---|---|---|
| CrownOS Repos | `pacman`, via the privileged helper | yes |
| AUR | `paru` or `yay`, elevating with `pkexec` | yes |
| Flathub | `flatpak --user` | no |
| Snap Store | `snap`, via the privileged helper | yes |

A source whose tool is missing is set up on first use (flatpak, snapd, paru),
then the app installs. Every operation runs in one queue, one at a time.
Adding a source means a file in `src/source/`, a `SourceKind` variant in
`crates/apps-database` and one arm in `source::backend`.

## Development

```sh
cargo build-index                                            # database/ → database/dist/v1
CROWNOS_APPSTORE_CATALOG=file://$PWD/database/dist cargo run # browse the local build
CROWNOS_APPSTORE_HELPER=$PWD/target/debug/crownos-appstore-helper cargo run

cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
cargo test -- --ignored probes_this_machine --nocapture      # probe the real package managers
```

The polkit policy in `data/` must be installed for the single
`auth_admin_keep` prompt per queue run; without it pkexec asks once per command.

## Releases

| Workflow | Does |
|---|---|
| `ci.yml` | format, lint and test on every push |
| `package.yml` | on a `v*` tag, publishes `crownos-appstore` and `snapd` packages on the release, which `crownos-iso` bakes into the ISO |
| `database.yml` | validates the database on every change; on `main`, publishes `database/dist` to `gh-pages` |
| `database-verify.yml` | every Monday, checks each listed package still exists upstream and opens an issue if not |

The database is served free from GitHub Pages
(`https://crown-os.github.io/crownos-appstore/v1/`), jsDelivr
(`https://cdn.jsdelivr.net/gh/Crown-OS/crownos-appstore@gh-pages/v1/`) and
`repo.crownos.org/apps`, tried in that order.

## Licenses

The code is MIT. The database metadata is CC0. Icons are the trademarks of
their owners and are redistributed as each project permits.
