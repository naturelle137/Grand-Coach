# Third-party files and libraries

Every third-party file (graphic, font, sound, copied code) and every runtime
dependency with a notable licence is recorded here **before** it enters the
repository.

Allowed are licences compatible with EUPL-1.2 (code) or CC BY-SA 4.0
(content). For code, e.g. MIT, Apache-2.0, BSD, Zlib, MPL-2.0 and the licences
listed in the EUPL appendix. For content: public domain, CC0, CC BY, CC BY-SA,
OFL (fonts). Not allowed: NC and ND licences, files of unclear origin.

Rust dependencies are listed in their own table below, checked by hand
against `cargo metadata`; an automated licence check is still to be set up.

| File / library | Source (URL) | Author | Licence | Changes |
|----------------|--------------|--------|---------|---------|
| –              | –            | –      | –       | –       |

## Rust dependencies

All are used unchanged from crates.io; versions are pinned in `Cargo.lock`.

| Crate | Used by | Licence |
|-------|---------|---------|
| `rand_chacha`, `rand_core` | `skat-core` (dealing, D-22) | MIT OR Apache-2.0 |
| `ppv-lite86` | via `rand_chacha` | MIT OR Apache-2.0 |
| `zerocopy` | via `ppv-lite86` | BSD-2-Clause OR Apache-2.0 OR MIT |
| `serde`, `serde_core`, `serde_derive` | `skat-core`, optional feature `serde` (D-25) | MIT OR Apache-2.0 |
| `proc-macro2`, `quote`, `syn` | via `serde_derive` (build time) | MIT OR Apache-2.0 |
| `unicode-ident` | via `proc-macro2`, `syn` | (MIT OR Apache-2.0) AND Unicode-3.0 |
| `serde_json`, `itoa`, `zmij` | tests only | MIT OR Apache-2.0 (`zmij`: MIT) |
| `memchr` | tests only, via `serde_json` | Unlicense OR MIT |
