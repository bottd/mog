use insta::{assert_yaml_snapshot, glob};
use mog_parser::parse;

#[test]
fn fixtures() {
    glob!("fixtures", "**/*.mg", |path| {
        assert_yaml_snapshot!(parse(&std::fs::read_to_string(path).unwrap()));
    });
}
