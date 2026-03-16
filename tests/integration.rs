use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_basic_golden() {
    let mut cmd = Command::cargo_bin("aquascope_svg").unwrap();
    cmd.arg("testdata/basic.golden").assert().success();
}
