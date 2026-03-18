use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_basic_json() {
    let mut cmd = Command::cargo_bin("aquascope_svg").unwrap();
    cmd.arg("testdata/json/basic.json").assert().success();
}

#[test]
fn test_all_testdata() {
    let test_files = [
        "testdata/json/basic.json",
        "testdata/json/box.json",
        "testdata/json/closure.json",
        "testdata/json/error.json",
        "testdata/json/nested_ref.json",
        "testdata/json/stackref.json",
        "testdata/json/tuple.json",
    ];

    for file in test_files {
        let mut cmd = Command::cargo_bin("aquascope_svg").unwrap();
        cmd.arg(file).assert().success();
    }
}
