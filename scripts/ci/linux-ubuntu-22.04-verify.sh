#!/usr/bin/env bash
# Compile/check the Rust workspace inside Docker ubuntu:22.04 so the
# binary is linked against glibc 2.35 / webkit2gtk-4.1.
#
# From the repo root:
#   bash scripts/ci/linux-ubuntu-22.04-verify.sh
#
# This does not produce a .deb or prove a Debian 12 install.
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"

if ! command -v docker >/dev/null 2>&1; then
  echo "docker is required to verify the Ubuntu 22.04 link target." >&2
  exit 1
fi

arch="$(uname -m)"
case "$arch" in
  arm64|aarch64) docker_platform=linux/arm64 ;;
  *) docker_platform=linux/amd64 ;;
esac

docker run --rm \
  --platform "$docker_platform" \
  -v "${root}:/src" \
  -w /src \
  ubuntu:22.04 \
  bash -lc '
    set -euo pipefail
    bash scripts/ci/linux-ubuntu-22.04-apt.sh
    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    . "$HOME/.cargo/env"
    cargo test -p crm-core --offline 2>/dev/null || cargo test -p crm-core
  '
