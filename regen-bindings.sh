#!/usr/bin/env bash
set -euo pipefail
export LIBCLANG_PATH="$(brew --prefix llvm)/lib"
export PATH="$(brew --prefix llvm)/bin:$PATH"
cargo build -p oxybox-sys --features bindgen
GEN="$(find target -type f -name bindings_gen.rs -print -quit)"
cp "$GEN" oxybox-sys/src/bindings.rs
