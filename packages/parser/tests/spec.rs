use insta::assert_yaml_snapshot;
use mog_parser::parse;

#[test]
fn spec() {
    let src = std::fs::read_to_string("../../spec.mg").unwrap();

    assert_yaml_snapshot!(parse(&src));
}
