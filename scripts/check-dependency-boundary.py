#!/usr/bin/env python3
"""Keep desktop-service dependencies out of every reusable UI target."""
import json
import subprocess

metadata = json.loads(subprocess.check_output([
    "cargo", "metadata", "--format-version", "1", "--locked",
], text=True))
packages = {package["id"]: package for package in metadata["packages"]}
nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
for member in metadata["workspace_members"]:
    pending = [member]
    visited = set()
    while pending:
        current = pending.pop()
        if current in visited:
            continue
        visited.add(current)
        if packages[current]["name"].startswith("knave-desktop"):
            raise SystemExit(f"{packages[member]['name']} depends on {packages[current]['name']}")
        pending.extend(nodes[current]["dependencies"])
print("Reusable UI dependency boundary passed (including dev dependencies).")
