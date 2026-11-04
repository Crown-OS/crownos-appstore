# App database

The catalog the App Store browses: one folder per app, compiled into a single
compressed index.

## Adding an app

1. Create `database/apps/<id>/app.toml` (the folder name must equal `id`):

   ```toml
   id = "firefox"
   name = "Firefox"
   summary = "Fast, private and free web browser"
   description = "Firefox is a free and open-source web browser…"
   category = "internet"   # internet communication development graphics multimedia office games education utilities system
   theme_color = "#ff7139"
   icon = "icon.png"        # .svg or .png, at least 128×128
   homepage = "https://www.mozilla.org/firefox"
   license = "MPL-2.0"
   keywords = ["browser", "web"]

   # In order of preference; one entry per kind.
   [[sources]]
   kind = "pacman"           # pacman | aur | flatpak | snap
   package = "firefox"

   [[sources]]
   kind = "snap"
   package = "code"
   classic = true            # snaps that need classic confinement
   ```

2. Add the icon. For apps on Flathub, `database/scripts/fetch-flathub-icon.sh <flatpak-id> database/apps/<id>/icon.png`
   downloads the largest official one.
3. Check it: `python3 database/scripts/verify-sources.py` and `cargo build-index`.
4. Open a pull request.

## What gets published

`cargo build-index` writes:

| Path | Contents |
|---|---|
| `database/dist/v1/index.bin.zst` | every app, [postcard](https://docs.rs/postcard)-encoded and zstd-compressed |
| `database/dist/v1/icons/<id>.png` | each icon as a 256×256 PNG |

The `v1` path is the schema version: an incompatible format change publishes
`v2` next to it, so stores already installed keep working.

Publishing and mirrors are described in the [top-level README](../README.md).
