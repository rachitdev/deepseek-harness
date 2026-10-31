#!/usr/bin/env python3
"""Per-service parity counts from docs/gui-endpoints.json and docs/parity-missing.txt."""
import json, collections
inv = json.load(open('docs/gui-endpoints.json'))['endpoints']
missing = {l.strip() for l in open('docs/parity-missing.txt') if l.strip() and not l.startswith('#')}
tot = collections.Counter(); miss = collections.Counter(); streams = collections.Counter()
for e in inv:
    s = e['endpoint'].split('/')[0]
    tot[s] += 1
    if e['endpoint'] in missing:
        miss[s] += 1
    if e['transport'] == 'stream':
        streams[s] += 1
print(f"total {len(inv)}  served {len(inv)-len(missing)}  missing {len(missing)}")
for s in sorted(tot):
    if miss[s]:
        print(f"{s:24} missing {miss[s]:2} of {tot[s]:2}  streams {streams[s]}")
