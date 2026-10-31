#!/bin/sh
set -eu

cd "$(dirname "$0")"

arch=$(uname -m)
case "$arch" in
    x86_64|amd64)  target=x86_64-unknown-linux-gnu ;;
    aarch64|arm64) target=aarch64-unknown-linux-gnu ;;
    *)             echo "Unsupported architecture: $arch" >&2; exit 1 ;;
esac

echo "Building $target..."
cargo build --release --target "$target" --bins

pkg="syncrab-$target"
mkdir -p release
rm -rf "release/$pkg"
mkdir "release/$pkg"
cp "target/$target/release/syncrab" \
   "target/$target/release/syncrab_b" \
   "target/$target/release/syncrab_w" "release/$pkg/"
tar -czf "release/$pkg.tar.gz" -C release "$pkg"
rm -rf "release/$pkg"

echo ""
echo "Created release/$pkg.tar.gz"
echo "Upload it on github.com: Releases -> Draft a new release -> attach the tarball"
