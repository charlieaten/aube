#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/../.." && pwd)"
out="${1:-"${repo_root}/target/beam-hex-package"}"

rm -rf "$out"
mkdir -p "$out"

cp -R "${repo_root}/beam/lib" "$out/lib"
cp "${repo_root}/beam/.formatter.exs" "$out/.formatter.exs"
cp "${repo_root}/beam/LICENSE" "$out/LICENSE"
cp "${repo_root}/beam/README.md" "$out/README.md"
cp "${repo_root}/beam/mix.exs" "$out/mix.exs"
cp "${repo_root}/beam/mix.lock" "$out/mix.lock"

shopt -s nullglob
for checksum in "${repo_root}"/beam/checksum-*.exs; do
  cp "$checksum" "$out/"
done
shopt -u nullglob

cp "${repo_root}/Cargo.toml" "$out/Cargo.toml"
cp "${repo_root}/Cargo.lock" "$out/Cargo.lock"

mkdir -p "$out/crates"
rsync -a \
  --exclude target \
  --exclude '.DS_Store' \
  "${repo_root}/crates/" \
  "$out/crates/"

printf '%s\n' "$out"
