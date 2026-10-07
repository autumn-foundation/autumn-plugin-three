#!/usr/bin/env bash
# Vendor Three.js into assets/. Run from the repository root.
# The script downloads the pinned upstream files to a temp dir and checks
# each sha384. Then it rewrites the addon import specifiers, checks that
# each rewrite matched one line, and moves the files into assets/.
# Keep the rewrites in sync with VendoredFile::rewrites in src/assets.rs.
# Needs GNU sed and coreutils (base64 -w0).
set -euo pipefail

VERSION="0.185.1"
BASE="https://cdn.jsdelivr.net/npm/three@${VERSION}"
OUT="assets"
TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

# upstream path | served name | pinned sha384
FILES=(
  "build/three.core.min.js|three.core.min.js|rx+KIp/9ptjArhnFAcpVoOc/ynktDsRtRJKIbC7YVKylEvFu8sgmzk9RmQ+CIV48"
  "build/three.module.min.js|three.module.min.js|QHQk1LzjJlJYNdthXjKCmffpDRZL3EqJ7LfqBzyKyvGgjAYM2ZVuYtFGg42NcAJ/"
  "examples/jsm/controls/OrbitControls.js|OrbitControls.js|4rziNxOBZKQ69i+w+f89KJ55TCYquwchVbByQwmaOeIOXdOU2PLDn3kOfXHwIJC9"
  "examples/jsm/loaders/GLTFLoader.js|GLTFLoader.js|3CnKaFWE2emo2DOUQi/yFm4SMemUgSZ9IAJe/V2pyJTw9KXWYSmR0MiX/7RoPyiJ"
  "examples/jsm/utils/BufferGeometryUtils.js|BufferGeometryUtils.js|05mkYituMJObxUkTK7xbW9SA45DEaxOge7FQUGrhW3dsFz0fckzykM6RMAPN26rD"
  "examples/jsm/utils/SkeletonUtils.js|SkeletonUtils.js|Pozn8j5+YFr3ak8Pm90ayqDrGYn/DV7vVs/YIIqzJhzeJT0LQksoS1fZQ5lfsYlw"
  "examples/jsm/environments/RoomEnvironment.js|RoomEnvironment.js|/H49oz0ZtMgJNgMZ+OhhuMuKBOsaiC3kY0/PZSvgJJXAOJJwmSBJjzlV5lJul3MB"
)

for entry in "${FILES[@]}"; do
  IFS='|' read -r upstream name pin <<<"${entry}"
  curl -fsSL "${BASE}/${upstream}" -o "${TMP}/${name}"
  actual="$(openssl dgst -sha384 -binary "${TMP}/${name}" | base64 -w0)"
  if [[ "${actual}" != "${pin}" ]]; then
    echo "sha384 mismatch: ${upstream}" >&2
    exit 1
  fi
done

# rewrite FILE OLD NEW: replace one exact line. Fail unless exactly one
# line matches before and after.
rewrite() {
  local file="${TMP}/$1" old="$2" new="$3"
  if [[ "$(grep -cxF "${old}" "${file}")" != 1 ]]; then
    echo "rewrite source not found once in $1: ${old}" >&2
    exit 1
  fi
  local escaped_old escaped_new
  escaped_old="$(printf '%s' "${old}" | sed 's/[.[\*^$/]/\\&/g')"
  escaped_new="$(printf '%s' "${new}" | sed 's/[&/\]/\\&/g')"
  sed -i "s/^${escaped_old}\$/${escaped_new}/" "${file}"
  if [[ "$(grep -cxF "${new}" "${file}")" != 1 ]]; then
    echo "rewrite failed in $1: ${new}" >&2
    exit 1
  fi
}

# Bare `three` and `../utils/` imports need an import map. The default
# Autumn CSP blocks inline import maps, so make the imports relative.
# See docs/adr/0002-addon-import-rewrites.md.
for name in OrbitControls.js GLTFLoader.js BufferGeometryUtils.js SkeletonUtils.js RoomEnvironment.js; do
  rewrite "${name}" "} from 'three';" "} from './three.module.min.js';"
done
rewrite GLTFLoader.js "import { toTrianglesDrawMode } from '../utils/BufferGeometryUtils.js';" \
  "import { toTrianglesDrawMode } from './BufferGeometryUtils.js';"
rewrite GLTFLoader.js "import { clone } from '../utils/SkeletonUtils.js';" \
  "import { clone } from './SkeletonUtils.js';"

for entry in "${FILES[@]}"; do
  IFS='|' read -r _ name _ <<<"${entry}"
  mv "${TMP}/${name}" "${OUT}/${name}"
done
echo "vendored three@${VERSION}"
