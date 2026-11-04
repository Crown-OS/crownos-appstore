#!/usr/bin/env python3
"""Checks that every package an app lists still exists upstream.

Prints one line per missing package and exits non-zero when any is missing.
"""

import json
import pathlib
import sys
import tomllib
import urllib.error
import urllib.parse
import urllib.request

APPS = pathlib.Path(__file__).resolve().parent.parent / "apps"

TIMEOUT_SECONDS = 20


def fetch(url, headers=None):
    request = urllib.request.Request(url, headers={"User-Agent": "crownos-appstore", **(headers or {})})
    try:
        with urllib.request.urlopen(request, timeout=TIMEOUT_SECONDS) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, b""


def pacman_exists(name):
    _, body = fetch("https://archlinux.org/packages/search/json/?name=" + urllib.parse.quote(name))
    return bool(json.loads(body or b"{}").get("results"))


def aur_exists(name):
    _, body = fetch("https://aur.archlinux.org/rpc/v5/info?arg[]=" + urllib.parse.quote(name))
    return json.loads(body or b"{}").get("resultcount") == 1


def flatpak_exists(name):
    return fetch("https://flathub.org/api/v2/appstream/" + urllib.parse.quote(name))[0] == 200


def snap_exists(name):
    url = "https://api.snapcraft.io/v2/snaps/info/" + urllib.parse.quote(name)
    return fetch(url, {"Snap-Device-Series": "16"})[0] == 200


CHECKS = {"pacman": pacman_exists, "aur": aur_exists, "flatpak": flatpak_exists, "snap": snap_exists}


def main():
    missing = []
    for manifest in sorted(APPS.glob("*/app.toml")):
        app = tomllib.loads(manifest.read_text())
        for source in app["sources"]:
            if not CHECKS[source["kind"]](source["package"]):
                missing.append(f"{app['id']}: {source['kind']} package {source['package']} not found")
    print("\n".join(missing) or "every source exists")
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
