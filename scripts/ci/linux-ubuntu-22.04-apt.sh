#!/usr/bin/env bash
# Install Ubuntu 22.04 (jammy) packages needed to compile and link 900CRM
# against glibc 2.35 and webkit2gtk-4.1. Run as root inside ubuntu:22.04.
set -euo pipefail

export DEBIAN_FRONTEND=noninteractive

apt-get update
apt-get install -y --no-install-recommends \
  build-essential \
  ca-certificates \
  curl \
  file \
  git \
  libayatana-appindicator3-dev \
  libgtk-3-dev \
  librsvg2-dev \
  libssl-dev \
  libwebkit2gtk-4.1-dev \
  libxdo-dev \
  patchelf \
  pkg-config \
  wget
