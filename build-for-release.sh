#!/bin/sh

# the script to build clekey ovr for release.

set -eu

cargo clean

export RUSTFLAGS="-C target-feature=+crt-static"
time cargo build \
  --release \
  --features openvr

# copy compiled binary to dest


mkdir -p dest
TARGETDIR="target/release"

cp "$TARGETDIR/clekey_ovr"* dest/
cp "$TARGETDIR"/build/clekey-ovr-rs-*/out/licenses.txt dest/
