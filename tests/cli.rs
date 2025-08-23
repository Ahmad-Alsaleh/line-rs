use assert_cmd::Command;
use assert_fs::{prelude::*, NamedTempFile};
use predicates::{ord::eq, str::ends_with};
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
        .arg(file.path())
        .assert()
        .success()
        .stdout("three\n");
}

#[test]
fn line_num_is_zero() {
    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=0")
        .arg("file")
        .assert()
        .failure()
        .stderr("Error: Line number can't be zero\n");
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
            "binrary file. Use `--allow-binary-files` to suppress this error\n",
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
        .stderr("Error: Line 4 is out of bound, input has 3 line(s)\n");
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
        .stderr("Error: Line -4 is out of bound, input has 3 line(s)\n");
}

#[test]
fn extract_first_line_in_negative() {
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
fn extract_last_line_in_negative() {
    let file = NamedTempFile::new("file").unwrap();
    file.write_str("one\ntwo\nthree").unwrap();

    Command::cargo_bin(BIN_NAME)
        .unwrap()
        .arg("-n=-1")
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
        .failure()
        .stderr("Error: Line 1 is out of bound, input has 0 line(s)\n");
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn single_number() {
//         let args = Cli::parse_from([".", "--line", "1", "file"]);
//         assert_eq!(*args.line_nums, [LineSelector::Single(Number::Positive(1))]);
//     }
//
//     #[test]
//     fn multiple_numbers() {
//         let args = Cli::parse_from([".", "--line", "1,2:3,4:4", "file"]);
//         assert_eq!(
//             *args.line_nums,
//             [
//                 LineSelector::Single(Number::Positive(1)),
//                 LineSelector::Range(Number::Positive(2), Number::Positive(3)),
//                 LineSelector::Single(Number::Positive(4)),
//             ]
//         );
//     }
//
//     #[test]
//     fn line_number_is_zero() {
//         let err = Cli::try_parse_from([".", "--line", "0", "file"]).unwrap_err();
//         // TODO: replace the below with a custom error. e.g.: LineSelectorError::ZeroLine
//         assert_eq!(
//             err.source().unwrap().to_string(),
//             "Line number can't be zero"
//         );
//     }
//
//     #[test]
//     fn space_around_comma() {
//         let args = Cli::parse_from([".", "--line", "1, 2,3 ,4 , 5", "file"]);
//         assert_eq!(
//             *args.line_nums,
//             [
//                 LineSelector::Single(Number::Positive(1)),
//                 LineSelector::Single(Number::Positive(2)),
//                 LineSelector::Single(Number::Positive(3)),
//                 LineSelector::Single(Number::Positive(4)),
//                 LineSelector::Single(Number::Positive(5)),
//             ]
//         )
//     }
//
//     #[test]
//     fn lower_bound_equals_upper_bound() {
//         let args = Cli::parse_from([".", "--line", "4:4", "file"]);
//         assert_eq!(*args.line_nums, [LineSelector::Single(Number::Positive(4))])
//     }
//
//     #[test]
//     fn lower_bound_more_than_upper_bound() {
//         let err = Cli::try_parse_from([".", "--line", "5:4", "file"]).unwrap_err();
//         // TODO: replace the below with a custom error. e.g.: LineSelectorError::InvertedRange
//         assert_eq!(
//             err.source().unwrap().to_string(),
//             "Lower bound can't be more than the upper bound: `5:4`"
//         );
//     }
// }
