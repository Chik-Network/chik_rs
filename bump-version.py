#!/usr/bin/env python3

# usage:
# bump-version <new-version-string> <previous-release-tag>

import os
import re
import sys
from pathlib import Path
from typing import Callable

v = sys.argv[1]
tag = sys.argv[2]

our_crates = [
    "crates/chik-bls",
    "crates/klvm-traits",
    "crates/chik-traits",
    "crates/chik_py_streamable_macro",
    "crates/chik_streamable_macro",
    "crates/chik-protocol",
    "crates/chik-tools",
    "crates/klvm-utils",
    "crates/klvm-derive",
    "crates/chik-puzzle-types",
    "crates/chik-client",
    "crates/chik-ssl",
    "crates/chik-consensus",
    "crates/chik-consensus/fuzz",
    "crates/chik-puzzle-types/fuzz",
    "crates/klvm-utils/fuzz",
]


def crates_with_changes() -> set[str]:
    ret = set()
    for c in our_crates:
        diff = os.popen(f"git diff {tag} -- {c}").read().strip()
        if len(diff) > 0:
            ret.add(c)
    # the python wheel is the top-level build target, we always want to bump its
    # version
    ret.add("wheel")
    return ret


def update_cargo(name: str, crates: set[str]) -> None:
    subst = ""
    with open(f"{name}/Cargo.toml") as f:
        for line in f:
            split = line.split()
            if split == []:
                subst += line
                continue

            if split[0] == "version" and name in crates:
                line = f'version = "{v}"\n'
            elif split[0] in crates and line.startswith(split[0] + " = "):
                line = re.sub(
                    'version = "([>=^]?)\d+\.\d+\.\d+"', f'version = "\\g<1>{v}"', line
                )
            subst += line

    with open(f"{name}/Cargo.toml", "w") as f:
        f.write(subst)


crates = crates_with_changes()
# always update the root crate (chik)
crates.add(".")
crates.add("chik")

crate_names = set([Path(n).name for n in crates])

print("bumping version of crates:")
for c in crate_names:
    print(f" - {c}")

for c in our_crates:
    update_cargo(c, crate_names)

update_cargo(".", crate_names)
update_cargo("wheel", crate_names)
