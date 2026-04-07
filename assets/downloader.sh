#!/bin/bash
# bladvak assets downloader

repo="https://raw.githubusercontent.com/Its-Just-Nans/bladvak/main/assets"
mkdir -p .github/workflows
for file in pages.yml release.yml rust.yml typos.yml; do
  curl -L -o ".github/workflows/$file" \
  "$repo/$file"
done
curl -L -O "$repo/.gitignore"
curl -L -O "$repo/rust-toolchain"
curl -L -O "$repo/index.html"
sed -i "s/BLADVAK_APP/$(basename "$PWD")/g" index.html

