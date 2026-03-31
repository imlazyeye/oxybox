#!/usr/bin/env bash

set -euo pipefail

REQUIRED_BINDGEN_VERSION="bindgen 0.72.1"
ACTUAL_BINDGEN_VERSION=$(bindgen --version)

if [ "$ACTUAL_BINDGEN_VERSION" != "$REQUIRED_BINDGEN_VERSION" ]; then
    echo "Error: requires $REQUIRED_BINDGEN_VERSION, got $ACTUAL_BINDGEN_VERSION"
    exit 1
fi

# run bindgen itself
bindgen ./oxybox-sys/vendor/box2d/include/box2d/box2d.h \
    --allowlist-function "b2.*" \
    --allowlist-type "b2.*" \
    --allowlist-var "b2.*" \
    --rust-edition 2024 \
    --rust-target 1.94 \
    --merge-extern-blocks \
    --output ./oxybox-sys/src/bindings.rs \
    -- -Ioxybox-sys/vendor/box2d/include