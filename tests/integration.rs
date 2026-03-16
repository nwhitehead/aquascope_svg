use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_basic_golden() {
    let mut cmd = Command::cargo_bin("aquascope_svg").unwrap();
    cmd.arg("testdata/basic.golden").assert().success();
}

#[test]
fn test_all_testdata() {
    let test_files = [
        "testdata/basic.golden",
        "testdata/box.golden",
        "testdata/closure.golden",
        "testdata/error.golden",
        "testdata/nested_ref.golden",
        "testdata/stackref.golden",
        "testdata/tuple.golden",
    ];

    for file in test_files {
        let mut cmd = Command::cargo_bin("aquascope_svg").unwrap();
        cmd.arg(file).assert().success();
    }
}
