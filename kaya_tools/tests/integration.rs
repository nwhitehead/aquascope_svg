use assert_cmd::prelude::*;
use std::process::Command;

const TEST_JSON_FILES: [&str; 15] = [
    "../testdata/json/array_0_error.json",
    "../testdata/json/basic.json",
    "../testdata/json/box.json",
    "../testdata/json/closure.json",
    "../testdata/json/error.json",
    "../testdata/json/for_loop_0.json",
    "../testdata/json/if_0.json",
    "../testdata/json/if_1.json",
    "../testdata/json/if_4.json",
    "../testdata/json/interior_move.json",
    "../testdata/json/linear_0.json",
    "../testdata/json/nested_ref.json",
    "../testdata/json/stackref.json",
    "../testdata/json/tuple.json",
    "../testdata/json/vec_0_error.json",
];

const TEST_KAYA_FILES: [&str; 15] = [
    "../testdata/bare/array_0_error.states",
    "../testdata/bare/basic.states",
    "../testdata/bare/box.states",
    "../testdata/bare/closure.states",
    "../testdata/bare/error.states",
    "../testdata/bare/for_loop_0.states",
    "../testdata/bare/if_0.states",
    "../testdata/bare/if_1.states",
    "../testdata/bare/if_4.states",
    "../testdata/bare/interior_move.states",
    "../testdata/bare/linear_0.states",
    "../testdata/bare/nested_ref.states",
    "../testdata/bare/stackref.states",
    "../testdata/bare/tuple.states",
    "../testdata/bare/vec_0_error.states",
];

#[test]
fn test_parse_states() {
    let mut cmd = Command::cargo_bin("render_kaya").unwrap();
    cmd.arg("--show-parse");
    cmd.arg("../testdata/test.states").assert().success();
}

#[test]
fn test_parse_all_states() {
    for file in TEST_KAYA_FILES {
        let mut cmd = Command::cargo_bin("render_kaya").unwrap();
        cmd.arg("--show-parse");
        cmd.arg(file).assert().success();
    }
}

#[test]
fn test_parse_failure() {
    let mut cmd = Command::cargo_bin("render_kaya").unwrap();
    cmd.arg("--show-parse");
    cmd.arg("../testdata/kaya/parse_error.kaya").assert().failure();

}

#[test]
fn test_basic_json() {
    let mut cmd = Command::cargo_bin("aquascope_json_to_kaya").unwrap();
    cmd.arg("../testdata/json/basic.json").assert().success();
}

#[test]
fn test_all_testdata_json() {
    for file in TEST_JSON_FILES {
        let mut cmd = Command::cargo_bin("aquascope_json_to_kaya").unwrap();
        cmd.arg(file).assert().success();
    }
}
