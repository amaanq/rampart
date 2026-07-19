#!/usr/bin/env bash
set -euo pipefail

browser=${1:-}
case "$browser" in
    chrome|firefox) ;;
    *)
        echo "usage: $0 chrome|firefox" >&2
        exit 2
        ;;
esac

extension_dir=$(cd -- "$(dirname -- "$0")" && pwd)
output_dir="$extension_dir/dist/$browser"
install -d "$output_dir" "$output_dir/icons"
for file in background.js content.js options.html options.js popup.html popup.js style.css; do
    install -m 0644 "$extension_dir/$file" "$output_dir/$file"
done
install -m 0644 "$extension_dir/icons/rampart.svg" "$output_dir/icons/rampart.svg"
install -m 0644 "$extension_dir/manifest.$browser.json" "$output_dir/manifest.json"

echo "$output_dir"
