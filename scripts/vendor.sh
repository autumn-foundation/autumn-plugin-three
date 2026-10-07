#!/usr/bin/env bash
# Vendor Three.js into assets/. Run from the repository root.
# Downloads the pinned upstream files, checks their sha384, then rewrites
# the addon import specifiers. The rewrites must stay in sync with
# VendoredFile::rewrites in src/assets.rs.
set -euo pipefail

VERSION="0.185.1"
BASE="https://cdn.jsdelivr.net/npm/three@${VERSION}"
OUT="assets"

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
  curl -fsSL "${BASE}/${upstream}" -o "${OUT}/${name}"
  actual="$(openssl dgst -sha384 -binary "${OUT}/${name}" | base64 -w0)"
  if [[ "${actual}" != "${pin}" ]]; then
    echo "sha384 mismatch: ${upstream}" >&2
    exit 1
  fi
done

# Bare `three` and `../utils/` imports do not resolve without an import
# map. The default Autumn CSP blocks inline import maps, so make them
# relative. See docs/adr/0002-addon-import-rewrites.md.
for name in OrbitControls.js GLTFLoader.js BufferGeometryUtils.js SkeletonUtils.js RoomEnvironment.js; do
  sed -i "s#^} from 'three';#} from './three.module.min.js';#" "${OUT}/${name}"
done
sed -i "s#^import { toTrianglesDrawMode } from '../utils/BufferGeometryUtils.js';#import { toTrianglesDrawMode } from './BufferGeometryUtils.js';#" "${OUT}/GLTFLoader.js"
sed -i "s#^import { clone } from '../utils/SkeletonUtils.js';#import { clone } from './SkeletonUtils.js';#" "${OUT}/GLTFLoader.js"

echo "vendored three@${VERSION}"
