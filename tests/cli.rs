//! End-to-end tests for the `mdfmt` binary.
#![expect(
    clippy::expect_used,
    reason = "test helpers outside #[test] fns panic on failure by design"
)]

mod common;

use std::{
    fs,
    path::Path,
    process::{Command, Output},
    time::{Duration, SystemTime},
};

/// Runs `mdfmt` with `args` and returns its output.
fn mdfmt(args: &[&Path]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdfmt"))
        .args(args)
        .env_remove("RUST_LOG")
        .output()
        .expect("run mdfmt")
}

/// Input fixture for a document that needs reformatting.
fn messy() -> String {
    common::fixture("document", "with_frontmatter", "in", "md")
}

/// Expected output for [`messy`].
fn tidy() -> String {
    common::fixture("document", "with_frontmatter", "out", "md")
}

#[test]
fn should_rewrite_file_in_place_when_it_needs_wrapping() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.md");
    fs::write(&file, messy()).unwrap();

    let out = mdfmt(&[&file]);

    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "stdout: {:?}", out.stdout);
    assert_eq!(fs::read_to_string(&file).unwrap(), tidy());
}

#[test]
fn should_leave_file_untouched_when_inside_obsidian_vault() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join(".obsidian")).unwrap();
    fs::create_dir(dir.path().join("notes")).unwrap();
    let file = dir.path().join("notes").join("a.md");
    fs::write(&file, messy()).unwrap();

    let out = mdfmt(&[&file]);

    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(fs::read_to_string(&file).unwrap(), messy());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("Obsidian"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn should_exit_1_and_format_the_rest_when_a_file_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.md");
    let file = dir.path().join("b.md");
    fs::write(&file, messy()).unwrap();

    let out = mdfmt(&[&missing, &file]);

    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(fs::read_to_string(&file).unwrap(), tidy());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("missing.md"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn should_not_write_file_when_already_formatted() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("c.md");
    fs::write(&file, tidy()).unwrap();
    let old = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_modified(old)
        .unwrap();

    let out = mdfmt(&[&file]);

    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(fs::metadata(&file).unwrap().modified().unwrap(), old);
}

#[test]
fn should_exit_2_when_no_path_is_given() {
    let out = mdfmt(&[]);

    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn should_leave_file_untouched_and_exit_1_when_reflow_is_unsafe() {
    let dir = tempfile::tempdir().unwrap();
    let unsafe_file = dir.path().join("rule.md");
    let unsafe_text = "Underscores y word word word word word word word \
word word word word word word _ _ _\n";
    fs::write(&unsafe_file, unsafe_text).unwrap();
    let file = dir.path().join("d.md");
    fs::write(&file, messy()).unwrap();

    let out = mdfmt(&[&unsafe_file, &file]);

    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(fs::read_to_string(&unsafe_file).unwrap(), unsafe_text);
    assert_eq!(fs::read_to_string(&file).unwrap(), tidy());
}

#[test]
fn should_list_file_and_exit_1_without_writing_when_check_finds_changes() {
    let dir = tempfile::tempdir().unwrap();
    let messy_file = dir.path().join("messy.md");
    fs::write(&messy_file, messy()).unwrap();
    let tidy_file = dir.path().join("tidy.md");
    fs::write(&tidy_file, tidy()).unwrap();

    let out = mdfmt(&[Path::new("--check"), &messy_file, &tidy_file]);

    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        format!("{}\n", messy_file.display())
    );
    assert_eq!(fs::read_to_string(&messy_file).unwrap(), messy());
}

#[test]
fn should_exit_0_and_print_nothing_when_check_finds_no_changes() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("tidy.md");
    fs::write(&file, tidy()).unwrap();

    let out = mdfmt(&[Path::new("--check"), &file]);

    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "stdout: {:?}", out.stdout);
}

#[test]
fn should_exit_1_and_still_report_other_files_when_check_hits_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.md");
    let messy_file = dir.path().join("messy.md");
    fs::write(&messy_file, messy()).unwrap();

    let out = mdfmt(&[Path::new("--check"), &missing, &messy_file]);

    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        format!("{}\n", messy_file.display())
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("missing.md"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
