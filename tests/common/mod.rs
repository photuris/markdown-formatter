//! Shared helpers for the golden-file tests.

use std::{fs, path::PathBuf};

/// Reads `tests/fixtures/<group>/<name>.<side>.<ext>` as a string.
pub fn fixture(group: &str, name: &str, side: &str, ext: &str) -> String {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "fixtures",
        group,
        &format!("{name}.{side}.{ext}"),
    ]
    .iter()
    .collect();

    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

/// Declares one test per fixture: `.in` must format to `.out`, and
/// `.out` must format to itself.
#[macro_export]
macro_rules! golden {
    ($group:literal, $ext:literal, $format:expr, [$($name:ident),+ $(,)?]) => {
        $(
            #[test]
            fn $name() {
                let input = common::fixture($group, stringify!($name), "in", $ext);
                let expected =
                    common::fixture($group, stringify!($name), "out", $ext);

                assert_eq!($format(&input), expected, "formatting .in");
                assert_eq!($format(&expected), expected, "formatting .out");
            }
        )+
    };
}
