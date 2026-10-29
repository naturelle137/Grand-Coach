#!/usr/bin/env python3
# SPDX-License-Identifier: EUPL-1.2
"""Carry ticked task-list items from an edited issue body over to the current one.

Usage: merge-checklist.py CURRENT EDITED > MERGED

Only `[ ]` -> `[x]` changes are taken over; items ticked in CURRENT stay
ticked. Exits with status 1 if EDITED differs from CURRENT in anything but
checkbox state, so an agent can never rewrite issue text this way.
"""

import re
import sys

BOX = re.compile(r"^(\s*(?:[-*+]|\d+[.)])\s+\[)([ xX])(\])")


def read_lines(path):
    with open(path, encoding="utf-8", newline="") as f:
        return f.read().splitlines(keepends=True)


def unticked(lines):
    out = [BOX.sub(r"\1 \3", line.rstrip("\r\n")) for line in lines]
    while out and not out[-1].strip():
        out.pop()
    return out


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    current = read_lines(sys.argv[1])
    edited = read_lines(sys.argv[2])
    if unticked(current) != unticked(edited):
        sys.exit("error: edited body differs from the current one beyond checkbox state")

    ticked = 0
    for i, line in enumerate(current):
        cur = BOX.match(line)
        new = BOX.match(edited[i]) if i < len(edited) else None
        if cur and new and cur.group(2) == " " and new.group(2) in "xX":
            line = BOX.sub(r"\1x\3", line, count=1)
            ticked += 1
        sys.stdout.write(line)
    print(f"{ticked} checklist item(s) newly ticked", file=sys.stderr)


if __name__ == "__main__":
    main()
