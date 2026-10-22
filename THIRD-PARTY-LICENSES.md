# Third-Party Licenses

RPCI IME itself is licensed under **GPL-3.0-or-later** (see [LICENSE](LICENSE)).
Third-party components keep their original licenses. Full texts are in
[LICENSES/](LICENSES/).

## Input method engine and schemas

| Component | License | Source | Notes |
|---|---|---|---|
| RIME / librime | BSD-3-Clause | https://github.com/rime/librime | Upstream library; dynamically linked through `librime-sys` |
| Debian/Ubuntu `librime1t64` | Includes GPL-3 components | Distribution package | System dependency, not bundled by RPCI |
| rime-wubi `wubi86.schema.yaml`, `wubi86.dict.yaml`, `wubi_pinyin.schema.yaml` | LGPL-3.0 | https://github.com/rime/rime-wubi | Copied into the user RIME data directory and packaged |
| rime-pinyin-simp `pinyin_simp.schema.yaml`, `pinyin_simp.dict.yaml` | Apache-2.0 | https://github.com/rime/rime-pinyin-simp | Fallback schema |

RPCI links against the system librime. The Debian/Ubuntu librime package
includes GPL-3 components such as `librime-octagram`; this is one reason the
combined RPCI distribution is GPL-3.0-or-later.

## Rust dependencies

Direct dependencies (see `Cargo.lock` for the complete tree):

| Component | License |
|---|---|
| `rime-api`, `librime-sys` | MIT OR Apache-2.0 |
| `smithay-client-toolkit` | MIT |
| `iced` | MIT |
| `cosmic-text` | MIT OR Apache-2.0 |
| `wayland-client`, `wayland-protocols`, `wayland-protocols-misc` | MIT |
| `xkbcommon` | MIT |
| `serde`, `toml`, `png`, `memmap2`, `libc` | MIT OR Apache-2.0 |

These licenses are compatible with GPL-3.0-or-later. Cargo dependencies are
not vendored as source in this repository.

## Project skills

`.agents/skills/` contains MIT-licensed skills from
[mattpocock/skills](https://github.com/mattpocock/skills). Their original
license is preserved at [.agents/skills/LICENSE](.agents/skills/LICENSE) and
[LICENSES/MIT-mattpocock-skills.txt](LICENSES/MIT-mattpocock-skills.txt).
They are source-repository material and are not part of the `.deb` payload.

## Documentation and screenshots

Project documentation and screenshots are distributed under
GPL-3.0-or-later, unless a file states otherwise. External product names and
trademarks belong to their respective owners.

## Not legal advice

This file is a practical inventory, not legal advice. If you redistribute
modified binaries or remove the system librime dependency, review the
applicable licenses with a qualified professional.
