#!/bin/sh
# Runs trunk (e.g. `./build.sh build --release` or `./build.sh serve --release`) with the C
# headers that grammar crates need. wasm32-unknown-unknown has no libc headers; newer grammar crates
# find Tree-sitter's own, older ones need them passed in, along with the gaps `wasm-include` fills.
set -eu
cd "$(dirname "$0")"

language_manifest=$(cargo metadata --format-version 1 |
    grep -o '"manifest_path":"[^"]*/crates/language/Cargo.toml"' |
    grep tree-sitter |
    head -n 1 |
    sed 's/"manifest_path":"//; s/"$//')
if [ -z "$language_manifest" ]; then
    echo "couldn't find the tree-sitter-language crate" >&2
    exit 1
fi
tree_sitter_headers="$(dirname "$language_manifest")/wasm/include"

# The C objects share the threaded build's memory, so they need the same features as the Rust code
# in .cargo/config.toml.
export CFLAGS_wasm32_unknown_unknown="-matomics -mbulk-memory -mmutable-globals -isystem $PWD/wasm-include -isystem $tree_sitter_headers"

# macOS's ar drops wasm objects it doesn't recognize, leaving the grammars' archives empty.
llvm_ar="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin/llvm-ar"
if [ ! -x "$llvm_ar" ]; then
    echo "llvm-ar is missing; install it with: rustup component add llvm-tools" >&2
    exit 1
fi
export AR_wasm32_unknown_unknown="$llvm_ar"
exec trunk "$@"
