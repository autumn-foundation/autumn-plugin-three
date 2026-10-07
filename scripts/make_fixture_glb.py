#!/usr/bin/env python3
"""Writes the test and demo models in static/models/.

broken.glb: the tile with an image that does not decode.

tile-ext.gltf (+ tile.bin, tile.png): the tile with its image at an
external URL. It tests the stock GLTFLoader image path.

tile.glb: a 1-unit plane ("Tile") with an embedded 2x2 PNG texture, and a
hidden copy ("Tile2") whose texture uses the same image with another
sampler. It tests images inside a GLB and the shared-image cache.

gem.glb: a test and demo model.

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


def png_2x2():
    """A 2x2 RGB PNG: red, green / blue, white."""
    import zlib
    rows = [b"\0" + bytes([255, 0, 0, 0, 255, 0]), b"\0" + bytes([0, 0, 255, 255, 255, 255])]
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 2, 2, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(b"".join(rows))) + chunk(b"IEND", b""))


def write_tile(path="static/models/tile.glb", png=None):
    pos = [(-0.5, -0.5, 0), (0.5, -0.5, 0), (0.5, 0.5, 0), (-0.5, 0.5, 0)]
    uv = [(0, 1), (1, 1), (1, 0), (0, 0)]
    idx = [0, 1, 2, 0, 2, 3]
    blobs = [
        b"".join(struct.pack("<3f", *p) for p in pos),
        b"".join(struct.pack("<2f", *t) for t in uv),
        b"".join(struct.pack("<H", i) for i in idx),
        png if png is not None else png_2x2(),
    ]
    views, offset, binary = [], 0, b""
    for blob in blobs:
        pad = (-len(blob)) % 4
        views.append({"buffer": 0, "byteOffset": offset, "byteLength": len(blob)})
        binary += blob + b"\0" * pad
        offset += len(blob) + pad
    doc = {
        "asset": {"version": "2.0", "generator": "autumn-plugin-three make_fixture_glb.py"},
        "scene": 0,
        "scenes": [{"nodes": [0, 1]}],
        "nodes": [{"name": "Tile", "mesh": 0}, {"name": "Tile2", "mesh": 1, "translation": [0, 0, -5]}],
        "meshes": [
            {"primitives": [{"attributes": {"POSITION": 0, "TEXCOORD_0": 1}, "indices": 2, "material": 0}]},
            {"primitives": [{"attributes": {"POSITION": 0, "TEXCOORD_0": 1}, "indices": 2, "material": 1}]},
        ],
        "materials": [
            {"pbrMetallicRoughness": {"baseColorTexture": {"index": 0}, "metallicFactor": 0.0}, "extensions": {"KHR_materials_unlit": {}}},
            {"pbrMetallicRoughness": {"baseColorTexture": {"index": 1}, "metallicFactor": 0.0}, "extensions": {"KHR_materials_unlit": {}}},
        ],
        "extensionsUsed": ["KHR_materials_unlit"],
        "textures": [{"source": 0, "sampler": 0}, {"source": 0, "sampler": 1}],
        "samplers": [{"magFilter": 9728, "minFilter": 9728}, {"magFilter": 9729, "minFilter": 9729}],
        "images": [{"bufferView": 3, "mimeType": "image/png", "extras": {"source": "fixture"}}],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": 4, "type": "VEC3", "min": [-0.5, -0.5, 0], "max": [0.5, 0.5, 0]},
            {"bufferView": 1, "componentType": 5126, "count": 4, "type": "VEC2"},
            {"bufferView": 2, "componentType": 5123, "count": 6, "type": "SCALAR"},
        ],
        "bufferViews": views,
        "buffers": [{"byteLength": len(binary)}],
    }
    chunk_json = json.dumps(doc, separators=(",", ":")).encode()
    chunk_json += b" " * ((-len(chunk_json)) % 4)
    payload = (struct.pack("<I4s", len(chunk_json), b"JSON") + chunk_json
               + struct.pack("<I4s", len(binary), b"BIN\0") + binary)
    with open(path, "wb") as f:
        f.write(struct.pack("<4sII", b"glTF", 2, 12 + len(payload)) + payload)
    print(f"wrote {path}")


write_tile()
# broken.glb: the tile with a PNG that does not decode.
write_tile("static/models/broken.glb", b"\x89PNG\r\n\x1a\nnot a real image")


def write_external_tile():
    pos = [(-0.5, -0.5, 0), (0.5, -0.5, 0), (0.5, 0.5, 0), (-0.5, 0.5, 0)]
    uv = [(0, 1), (1, 1), (1, 0), (0, 0)]
    idx = [0, 1, 2, 0, 2, 3]
    blobs = [
        b"".join(struct.pack("<3f", *p) for p in pos),
        b"".join(struct.pack("<2f", *t) for t in uv),
        b"".join(struct.pack("<H", i) for i in idx),
    ]
    views, offset, binary = [], 0, b""
    for blob in blobs:
        pad = (-len(blob)) % 4
        views.append({"buffer": 0, "byteOffset": offset, "byteLength": len(blob)})
        binary += blob + b"\0" * pad
        offset += len(blob) + pad
    doc = {
        "asset": {"version": "2.0", "generator": "autumn-plugin-three make_fixture_glb.py"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"name": "Tile", "mesh": 0}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "TEXCOORD_0": 1}, "indices": 2, "material": 0}]}],
        "materials": [{"pbrMetallicRoughness": {"baseColorTexture": {"index": 0}, "metallicFactor": 0.0}, "extensions": {"KHR_materials_unlit": {}}}],
        "extensionsUsed": ["KHR_materials_unlit"],
        "textures": [{"source": 0, "sampler": 0}],
        "samplers": [{"magFilter": 9728, "minFilter": 9728}],
        "images": [{"uri": "tile.png"}],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": 4, "type": "VEC3", "min": [-0.5, -0.5, 0], "max": [0.5, 0.5, 0]},
            {"bufferView": 1, "componentType": 5126, "count": 4, "type": "VEC2"},
            {"bufferView": 2, "componentType": 5123, "count": 6, "type": "SCALAR"},
        ],
        "bufferViews": views,
        "buffers": [{"byteLength": len(binary), "uri": "tile.bin"}],
    }
    with open("static/models/tile-ext.gltf", "w") as f:
        json.dump(doc, f, indent=1)
    with open("static/models/tile.bin", "wb") as f:
        f.write(binary)
    with open("static/models/tile.png", "wb") as f:
        f.write(png_2x2())
    print("wrote static/models/tile-ext.gltf, tile.bin, tile.png")


write_external_tile()
