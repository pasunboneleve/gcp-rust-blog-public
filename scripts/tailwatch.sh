#!/usr/bin/env bash
#
# SCRIPT: tailwatch.sh
# AUTHOR: pasunboneleve
# DATE: 2025-12-12
# DESCRIPTION: watch tailwind.css and create a new
# static CSS bundle for the browser to use.
#
# USAGE: ./tailwatch.sh
#
set -euox pipefail

tailwindcss -i tailwind.css \
            -o content/static/tailwind.css \
            --config tailwind.config.js --watch
