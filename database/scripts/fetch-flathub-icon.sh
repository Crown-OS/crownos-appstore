#!/usr/bin/env bash
# Downloads the largest Flathub icon of a flatpak app.
#
#   fetch-flathub-icon.sh FLATPAK_ID DESTINATION.png

set -euo pipefail

id=${1:?usage: fetch-flathub-icon.sh FLATPAK_ID DESTINATION.png}
destination=${2:?missing DESTINATION.png}

mapfile -t urls < <(curl -fsS "https://flathub.org/api/v2/appstream/$id" |
    jq -r '((.icons // []) | sort_by(-(.scale // 1)) | .[].url), (.icon // empty)')
for url in "${urls[@]}"; do
    curl -fsS -o "$destination" "$url" 2>/dev/null && exit 0
done
printf '%s has no downloadable icon on Flathub\n' "$id" >&2
exit 1
