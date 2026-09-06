#!/bin/sh
# Puts a shallow checkout of Servo at ../servo, at the commit recorded in SERVO_COMMIT.
# senga's Cargo.toml points there as a path dependency.
set -eu
here=$(cd "$(dirname "$0")/.." && pwd)
dest=${SERVO_DIR:-"$here/../servo"}
commit=$(cat "$here/SERVO_COMMIT")

if [ ! -d "$dest/.git" ]; then
  git clone --depth 1 https://github.com/servo/servo.git "$dest"
fi
cd "$dest"
if [ "$(git rev-parse HEAD)" != "$commit" ]; then
  # A shallow clone only has its own tip; fetch the pinned commit by hash.
  git fetch --depth 1 origin "$commit"
  git checkout -q --detach "$commit"
fi
echo "servo at $(git rev-parse --short HEAD) in $dest"
