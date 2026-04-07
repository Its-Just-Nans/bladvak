#!/bin/bash
# bladvak assets downloader

repo="https://raw.githubusercontent.com/Its-Just-Nans/bladvak/main/bladvak-cli/assets"
app_name="$(basename "$PWD")"

mkdir -p .github/workflows
for file in pages.yml release.yml rust.yml typos.yml; do
  curl -L -o ".github/workflows/$file" \
  "$repo/$file"
done

curl -L -O "$repo/check.sh"
curl -L -O "$repo/.gitignore"
curl -L -O "$repo/rust-toolchain"
curl -L -O "$repo/index.html"
sed -i "s/BLADVAK_APP/$app_name/g" index.html

mkdir -p assets
curl -L -O "$repo/sw.js"
curl -L -O "$repo/manifest.json"
sed -i "s/BLADVAK_APP/$app_name/g" sw.js
sed -i "s/BLADVAK_APP/$app_name/g" manifest.json

