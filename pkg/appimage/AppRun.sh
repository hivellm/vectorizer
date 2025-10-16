#!/bin/bash
# AppImage launcher script for Vectorizer

HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"

# Set default config if not specified
if [ -z "$VECTORIZER_CONFIG" ]; then
    export VECTORIZER_CONFIG="${HERE}/usr/share/config.yml"
fi

# Run vectorizer
exec "${HERE}/usr/bin/vectorizer" "$@"

