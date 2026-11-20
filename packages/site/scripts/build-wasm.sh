#!/usr/bin/env bash
# Build crates/lynter-wasm into packages/site/public/wasm (web target).
# Needs: rustup target add wasm32-unknown-unknown, cargo install wasm-bindgen-cli --version 0.2.130 --locked.
set -euo pipefail
root="$(cd "$(dirname "$0")/../../.." && pwd)"
out="$root/packages/site/public/wasm"
want="0.2.130"
have="$(wasm-bindgen --version | awk '{print $2}')"
[ "$have" = "$want" ] || { echo "wasm-bindgen-cli $want required, found $have" >&2; exit 1; }
export CARGO_PROFILE_RELEASE_OPT_LEVEL=z CARGO_PROFILE_RELEASE_LTO=true \
  CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_PROFILE_RELEASE_PANIC=abort \
  CARGO_PROFILE_RELEASE_STRIP=debuginfo
cd "$root"
cargo build -p lynter-wasm --release --target wasm32-unknown-unknown
rm -rf "$out"
wasm-bindgen --target web --no-typescript --out-dir "$out" \
  "${CARGO_TARGET_DIR:-$root/target}/wasm32-unknown-unknown/release/lynter_wasm.wasm"
# wasm-opt is deliberately skipped: binaryen 108 shrinks the externref table the glue fills, and saves only 5 percent.
# .mjs keeps the glue out of the repository's *.js ignore rule and marks it as ESM.
mv "$out/lynter_wasm.js" "$out/lynter_wasm.mjs"
wasm="$out/lynter_wasm_bg.wasm"
raw="$(wc -c <"$wasm")"
gz="$(gzip -9 <"$wasm" | wc -c)"
limit=900000
echo "raw $raw bytes, gzip $gz bytes (limit $limit)"
[ "$gz" -le "$limit" ] || { echo "gzip size $gz exceeds $limit bytes" >&2; exit 1; }
