use insta::{assert_yaml_snapshot, glob};
use mog_parser::{parse, parse_unfolded};

#[test]
fn fixtures() {
    glob!("fixtures", "**/*.mg", |path| {
        assert_yaml_snapshot!(parse_unfolded(&std::fs::read_to_string(path).unwrap()));
    });
}

// folding only changes trees that hold attribute nodes
#[test]
fn folded() {
    glob!("fixtures", "attribute-*.mg", |path| {
        assert_yaml_snapshot!(parse(&std::fs::read_to_string(path).unwrap()));
    });
}
