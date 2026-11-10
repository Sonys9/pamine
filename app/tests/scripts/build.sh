#!/usr/bin/env bash
# Rebuilds `test_script.tar` from the `test_script/` sources next to it.
#
# The archive is committed because the tests `include_bytes!` it, so it has to
# exist at compile time. Run this after editing anything under `test_script/`.
#
# The archive keeps a single top-level `test_script/` directory: the loader
# unwraps it, so the paths inside the script stay relative to the script root.
set -euo pipefail

cd "$(dirname "$0")"

tar --create --file test_script.tar test_script

echo "Rebuilt $(pwd)/test_script.tar"
tar --list --file test_script.tar
