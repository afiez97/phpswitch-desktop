#!/usr/bin/env bash
# Runs the phpswitch bats suite. Requires bats-core:
#   Ubuntu/Debian: sudo apt-get install -y bats
#   Any platform:  git clone https://github.com/bats-core/bats-core.git && ./bats-core/install.sh /usr/local
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

if ! command -v bats &>/dev/null; then
    echo "bats-core is not installed — see the comment at the top of this script." >&2
    exit 1
fi

exec bats phpswitch.bats
