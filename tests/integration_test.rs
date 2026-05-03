use apsb26_43_pdf_read::reformat_js;

#[test]
fn integer_constants() {
    let result = reformat_js(
        "test.js",
        "1 + 1".to_string());
    assert_eq!(result.trim_end(), "2;");
}

#[test]
fn producing_undefined() {
    let result = reformat_js(
        "test.js",
        "[][[]]".to_string());
    assert_eq!(result.trim_end(), "undefined;");
}

#[test]
fn producing_nan() {
    let result = reformat_js(
        "test.js",
        "+{}".to_string());
    assert_eq!(result.trim_end(), "(\"NaN\");");
}

#[test]
fn producing_false() {
    let result = reformat_js(
        "test.js",
        "![]".to_string());
    assert_eq!(result.trim_end(), "false;");
}

#[test]
fn producing_true() {
    let result = reformat_js(
        "test.js",
        "!![]".to_string());
    assert_eq!(result.trim_end(), "true;");
}