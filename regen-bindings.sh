#!/usr/bin/env bash
set -euo pipefail

git submodule init;
git submodule update;

bindgen ./oxybox-sys/vendor/box2d/include/box2d/box2d.h \
    --allowlist-function "b2.*" \
    --allowlist-type "b2.*" \
    --allowlist-var "b2.*" \
    --rust-edition 2024 \
    --rust-target 1.94 \
    --merge-extern-blocks \
    --output ./oxybox-sys/src/bindings.rs \
    -- -Ioxybox-sys/vendor/box2d/include