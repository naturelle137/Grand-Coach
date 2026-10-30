// SPDX-License-Identifier: EUPL-1.2
//! `THIRD-PARTY-NOTICES` lists exactly the third-party crates in `Cargo.lock`.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn field<'a>(block: &'a str, key: &str) -> Option<&'a str> {
    block.lines().find_map(|line| {
        line.strip_prefix(key)?
            .strip_prefix(" = \"")?
            .strip_suffix('"')
    })
}

#[test]
fn notices_match_cargo_lock() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let lock = fs::read_to_string(root.join("Cargo.lock")).expect("Cargo.lock");
    let expected: BTreeSet<String> = lock
        .split("[[package]]")
        .skip(1)
        .filter(|block| field(block, "source").is_some())
        .map(|block| {
            format!(
                "{} {}",
                field(block, "name").expect("name"),
                field(block, "version").expect("version")
            )
        })
        .collect();

    let notices =
        fs::read_to_string(root.join("THIRD-PARTY-NOTICES")).expect("THIRD-PARTY-NOTICES");
    let listed: BTreeSet<String> = notices
        .lines()
        .filter_map(|line| line.strip_prefix("== ")?.strip_suffix(" =="))
        .filter(|heading| !heading.starts_with("Appendix"))
        .map(str::to_owned)
        .collect();

    assert_eq!(
        listed, expected,
        "THIRD-PARTY-NOTICES is out of date; run `cargo run --locked -p notices`"
    );
}
