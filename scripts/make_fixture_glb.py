#!/usr/bin/env python3
"""Writes static/models/gem.glb: a test and demo model.

A 4-unit box. Its node is at (10, 0, 0) with an orange PBR material and one
animation clip, "Spin" (2 s, one turn on Y). The size and offset test
`data-three-fit`; the clip tests `data-three-clip`. Run from the repo root.
"""
import json
import math
import struct

H = 2.0  # half size
CX = 10.0  # center offset on X

faces = [  # normal, then four corners (counter-clockwise from outside)
    ((1, 0, 0), [(H, -H, H), (H, -H, -H), (H, H, -H), (H, H, H)]),
    ((-1, 0, 0), [(-H, -H, -H), (-H, -H, H), (-H, H, H), (-H, H, -H)]),
    ((0, 1, 0), [(-H, H, H), (H, H, H), (H, H, -H), (-H, H, -H)]),
    ((0, -1, 0), [(-H, -H, -H), (H, -H, -H), (H, -H, H), (-H, -H, H)]),
    ((0, 0, 1), [(-H, -H, H), (H, -H, H), (H, H, H), (-H, H, H)]),
    ((0, 0, -1), [(H, -H, -H), (-H, -H, -H), (-H, H, -H), (H, H, -H)]),
]
positions, normals, indices = [], [], []
for normal, corners in faces:
    base = len(positions)
    for x, y, z in corners:
        positions.append((x, y, z))
        normals.append(normal)
    indices += [base, base + 1, base + 2, base, base + 2, base + 3]

times = [0.0, 0.5, 1.0, 1.5, 2.0]
quats = []
for t in times:
    half = math.pi * t / 2.0  # angle / 2, angle = pi * t
    quats.append((0.0, math.sin(half), 0.0, math.cos(half)))

blobs = [
    b"".join(struct.pack("<3f", *p) for p in positions),
    b"".join(struct.pack("<3f", *n) for n in normals),
    b"".join(struct.pack("<H", i) for i in indices),
    b"".join(struct.pack("<f", t) for t in times),
    b"".join(struct.pack("<4f", *q) for q in quats),
]
views, offset, binary = [], 0, b""
for blob in blobs:
    pad = (-len(blob)) % 4
    views.append({"buffer": 0, "byteOffset": offset, "byteLength": len(blob)})
    binary += blob + b"\0" * pad
    offset += len(blob) + pad

gltf = {
    "asset": {"version": "2.0", "generator": "autumn-plugin-three make_fixture_glb.py"},
    "scene": 0,
    "scenes": [{"nodes": [0]}],
    "nodes": [{"name": "Gem", "mesh": 0, "translation": [CX, 0.0, 0.0]}],
    "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1}, "indices": 2, "material": 0}]}],
    "materials": [{"pbrMetallicRoughness": {"baseColorFactor": [1.0, 0.45, 0.1, 1.0], "metallicFactor": 0.1, "roughnessFactor": 0.5}}],
    "animations": [{
        "name": "Spin",
        "samplers": [{"input": 3, "output": 4, "interpolation": "LINEAR"}],
        "channels": [{"sampler": 0, "target": {"node": 0, "path": "rotation"}}],
    }],
    "accessors": [
        {"bufferView": 0, "componentType": 5126, "count": len(positions), "type": "VEC3",
         "min": [-H, -H, -H], "max": [H, H, H]},
        {"bufferView": 1, "componentType": 5126, "count": len(normals), "type": "VEC3"},
        {"bufferView": 2, "componentType": 5123, "count": len(indices), "type": "SCALAR"},
        {"bufferView": 3, "componentType": 5126, "count": len(times), "type": "SCALAR", "min": [0.0], "max": [2.0]},
        {"bufferView": 4, "componentType": 5126, "count": len(quats), "type": "VEC4"},
    ],
    "bufferViews": views,
    "buffers": [{"byteLength": len(binary)}],
}
json_chunk = json.dumps(gltf, separators=(",", ":")).encode()
json_chunk += b" " * ((-len(json_chunk)) % 4)
body = (struct.pack("<I4s", len(json_chunk), b"JSON") + json_chunk
        + struct.pack("<I4s", len(binary), b"BIN\0") + binary)
with open("static/models/gem.glb", "wb") as f:
    f.write(struct.pack("<4sII", b"glTF", 2, 12 + len(body)) + body)
print("wrote static/models/gem.glb")
