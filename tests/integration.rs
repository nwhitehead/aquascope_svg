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
        "testdata/json/apply_curve_error.json",
        "testdata/json/array_0_error.json",
        "testdata/json/basic.json",
        "testdata/json/box.json",
        "testdata/json/closure.json",
        "testdata/json/error.json",
        "testdata/json/for_loop_0.json",
        "testdata/json/if_0.json",
        "testdata/json/if_1.json",
        "testdata/json/if_2.json",
        "testdata/json/if_3.json",
        "testdata/json/if_4.json",
        "testdata/json/illegal_promotion_0.json",
        "testdata/json/interior_move_1.json",
        "testdata/json/interior_move.json",
        "testdata/json/linear_0.json",
        "testdata/json/match_0.json",
        "testdata/json/match_1.json",
        "testdata/json/nested-ref.json",
        "testdata/json/nested_ref.json",
        "testdata/json/stackref.json",
        "testdata/json/struct_with_dtor.json",
        "testdata/json/tuple.json",
        "testdata/json/vec_0_error.json",
        "testdata/json/vec_0.json",
        "testdata/json/while_loop_0.json",
    ];

    for file in test_files {
        let mut cmd = Command::cargo_bin("aquascope_svg").unwrap();
        cmd.arg(file).assert().success();
    }
}
