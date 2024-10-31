use assert_cmd::Command;

#[test]
fn test_hello() {
    let mut cmd = Command::cargo_bin("rvem").unwrap();
    let assert = cmd.arg("tests/data/hello").assert();

    assert.success().code(0).stdout("Hello World!\n");
}

#[test]
#[allow(non_snake_case)]
fn test_complexMul() {
    let mut cmd = Command::cargo_bin("rvem").unwrap();
    let assert = cmd.arg("tests/data/complexMul").assert();

    assert.success().code(0).stdout("-7 + i* 19");
}

#[test]
fn test_fac() {
    let mut cmd = Command::cargo_bin("rvem").unwrap();
    let assert = cmd.arg("tests/data/fac").assert();

    assert.success().code(0).stdout("120");
}

#[test]
fn test_fib() {
    let mut cmd = Command::cargo_bin("rvem").unwrap();
    let assert = cmd.arg("tests/data/fib").assert();

    assert.success().code(0).stdout("267914296");
}

#[test]
fn test_strlen() {
    let mut cmd = Command::cargo_bin("rvem").unwrap();
    let assert = cmd.arg("tests/data/strlen").assert();

    assert.success().code(0).stdout("44");
}
