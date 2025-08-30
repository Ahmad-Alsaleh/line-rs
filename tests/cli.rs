use assert_cmd::Command;
use assert_fs::{NamedTempFile, TempDir, prelude::*};
use predicates::{
    ord::eq,
    str::{ends_with, starts_with},
};
use std::{fs::Permissions, os::unix::fs::PermissionsExt};

const BIN_NAME: &str = "line";

#[test]
fn extract_line_in_middle() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n")
        .arg("1")
        .arg("-p")
        .arg(file.path())
        .assert()
        .success()
        .stdout("one\n");
}

#[test]
fn extract_last_line() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("--line")
        .arg("3")
        .arg("-p")
        .arg(file.path())
        .assert()
        .success()
        .stdout("three");

    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree\n").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("--line")
        .arg("3")
        .arg("-p")
        .arg(file.path())
        .assert()
        .success()
        .stdout("three\n");
}

#[test]
fn line_num_is_zero() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=0")
        .arg(file.path())
        .assert()
        .failure()
        .stderr(starts_with(
            "error: invalid value '0' for '--line <LINE_SELECTORS>': Zero is not allowed. Use \
            positive numbers (1, 2, ...) or negative numbers (-1, -2, ...) for backward counting",
        ));
}

#[test]
fn rejects_binary_file_without_flag() {
    let file = NamedTempFile::new("file").unwrap();
    let content = [0, 146, 150, b'\n', 0, 158, 147, b'\n', 151, 0, 167];
    file.write_binary(&content).unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=1")
        .arg(file.path())
        .assert()
        .failure()
        .stderr(predicates::str::ends_with(
            "binary file (use --allow-binary-files to override)\n",
        ));
}

#[test]
fn accepts_binary_file_with_flag() {
    let file = NamedTempFile::new("file").unwrap();
    let content = [0, 146, 150, b'\n', 0, 158, 147, b'\n', 151, 0, 167];
    file.write_binary(&content).unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=2")
        .arg("-p")
        .arg(file.path())
        .arg("--allow-binary-files")
        .assert()
        .success()
        .stdout(eq(&content[4..8]));
}

#[test]
fn file_does_not_exist() {
    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=1")
        .arg("file")
        .assert()
        .failure()
        .stderr(ends_with("No such file or directory (os error 2)\n"));
}

#[test]
fn no_read_permesions() {
    let file = NamedTempFile::new("file").unwrap();
    file.touch().unwrap();

    std::fs::set_permissions(file.path(), Permissions::from_mode(0o200)).unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=1")
        .arg(file.path())
        .assert()
        .failure()
        .stderr(ends_with("Permission denied (os error 13)\n"));
}

#[test]
fn line_too_large() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=4")
        .arg(file.path())
        .assert()
        .failure()
        .stderr("Error: Invalid line selector: 4\n\nCaused by:\n    Line 4 is out of range (input has only 3 line(s))\n");
}

#[test]
fn line_too_small() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=-4")
        .arg(file.path())
        .assert()
        .failure()
        .stderr("Error: Invalid line selector: -4\n\nCaused by:\n    Line -4 is out of range (input has only 3 line(s))\n");
}

#[test]
fn extract_first_line_in_negative() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=-3")
        .arg("-p")
        .arg(file.path())
        .assert()
        .success()
        .stdout("one\n");
}

#[test]
fn extract_last_line_in_negative() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=-1")
        .arg("-p")
        .arg(file.path())
        .assert()
        .success()
        .stdout("three");
}

#[test]
fn extract_middle_line_in_negative() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=-2")
        .arg("-p")
        .arg(file.path())
        .assert()
        .success()
        .stdout("two\n");
}

#[test]
fn empty_file() {
    let file = NamedTempFile::new("file").unwrap();
    file.touch().unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=1")
        .arg(file.path())
        .assert()
        .success()
        .stdout("--- EMPTY FILE ---\n");

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=1")
        .arg("--plain")
        .arg(file.path())
        .assert()
        .success()
        .stdout("");
}

#[test]
fn without_plain_flag() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=-3")
        .arg(file.path())
        .assert()
        .success()
        .stdout("one\n");
}

#[test]
fn unbounded_start() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=:-2")
        .arg("--plain")
        .arg(file.path())
        .assert()
        .success()
        .stdout("one\ntwo\n");
}

#[test]
fn unbounded_end() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=-2:")
        .arg("--plain")
        .arg(file.path())
        .assert()
        .success()
        .stdout("two\nthree");
}

#[test]
fn unbounded_start_and_end() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=:")
        .arg("--plain")
        .arg(file.path())
        .assert()
        .success()
        .stdout("one\ntwo\nthree");
}

#[test]
fn ranges_with_single_lines() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree\n").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n")
        .arg("1,1:3,1:1")
        .arg("--plain")
        .arg(file.path())
        .assert()
        .success()
        .stdout("one\none\ntwo\nthree\none\n");
}

#[test]
fn space_around_comma() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree\n").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n")
        .arg("1, 2,3 ,2 , 1")
        .arg("--plain")
        .arg(file.path())
        .assert()
        .success()
        .stdout("one\ntwo\nthree\ntwo\none\n");
}

#[test]
fn start_more_than_end() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree\n").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n")
        .arg("3:2")
        .arg(file.path())
        .assert()
        .failure()
        .stderr(
            "Error: Invalid line selector: 3:2\n\nCaused by:\n    The start of the range can't \
            be more than its end when the step is positive\n",
        );

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n")
        .arg("3:2:2")
        .arg(file.path())
        .assert()
        .failure()
        .stderr(
            "Error: Invalid line selector: 3:2:2\n\nCaused by:\n    The start of the range \
            can't be more than its end when the step is positive\n",
        );

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n")
        .arg("1:3:-1")
        .arg(file.path())
        .assert()
        .failure()
        .stderr(
            "Error: Invalid line selector: 1:3:-1\n\nCaused by:\n    The start of the range \
            can't be less than its end when the step is negative\n",
        );
}

#[test]
fn step_is_zero() {
    let file = NamedTempFile::new("file").unwrap();
    file.touch().unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n")
        .arg("3:2:0")
        .arg(file.path())
        .assert()
        .failure()
        .stderr(starts_with(
            "error: invalid value '3:2:0' for '--line <LINE_SELECTORS>': Zero is not allowed. Use \
            positive numbers (1, 2, ...) or negative numbers (-1, -2, ...) for backward counting",
        ));
}

#[test]
fn empty_line_selector() {
    let file = NamedTempFile::new("file").unwrap();
    file.touch().unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n")
        .arg("")
        .arg(file.path())
        .assert()
        .failure()
        .stderr(starts_with(
            "error: invalid value '' for '--line <LINE_SELECTORS>': Line number can't be empty",
        ));
}

#[test]
fn start_less_than_end_with_step() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree\n").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n")
        .arg("1:2:1")
        .arg("-p")
        .arg(file.path())
        .assert()
        .success()
        .stdout("one\ntwo\n");
}

#[test]
fn start_equals_end_with_step() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree\n").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n")
        .arg("2:2:1")
        .arg("-p")
        .arg(file.path())
        .assert()
        .success()
        .stdout("two\n");
}

#[test]
fn negative_step() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree\n").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n")
        .arg("2:1:-1")
        .arg("-p")
        .arg(file.path())
        .assert()
        .success()
        .stdout("two\none\n");
}

#[test]
fn not_a_file() {
    let file = TempDir::new().unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n")
        .arg("1")
        .arg(file.path())
        .assert()
        .failure()
        .stderr(ends_with("not a file\n"));
}
