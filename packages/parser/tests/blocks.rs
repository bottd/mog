//! Block-level parsing tests.
//!
//! Examples are copied verbatim from spec.mg. Each test cites the section
//! it was pulled from so the spec and tests stay in sync.
//!
//! Tests are `#[ignore]`'d while `mog_parser::parse` is a `todo!()`. Remove
//! the ignore attribute once the parser returns meaningful output, then run
//! `cargo insta review` (cargo install cargo-insta) to accept snapshots.

use insta::assert_yaml_snapshot;
use mog_parser::parse;

#[test]
fn bom() {
    // §1.1 — a leading U+FEFF is stripped and not considered content,
    // so the BOM-prefixed doc must parse identically to the bare one.
    let bare = parse("# My heading\n");
    let with_bom = parse("\u{FEFF}# My heading\n");
    assert_eq!(bare, with_bom);
    assert_yaml_snapshot!(bare);
}

#[test]
fn whitespace_equivalence() {
    // §1.2 — trailing whitespace on the second line is intentional.
    let plain = parse("# My heading\n");
    let padded = parse("  # My heading  \n");
    assert_eq!(plain, padded);
    assert_yaml_snapshot!(plain);
}

#[test]
fn metadata() {
    // §1.4 Metadata
    let examples = [r#"--
title "My Document"
authors "John" "Jane"
date "2026-04-15"
version 1
--
"#];
    assert_yaml_snapshot!(examples.map(parse));
}

#[test]
#[ignore = "parser unimplemented"]
fn nesting() {
    // §2.1 Nesting
    let examples = [r#"# Heading 1
## Heading 2
### Heading 3
 ## Heading 2 with leading whitespace
  # Heading 1 with leading whitespace
"#];
    assert_yaml_snapshot!(examples.map(parse));
}

#[test]
#[ignore = "parser unimplemented"]
fn blocks() {
    // §2.2 — both forms should parse equivalently.
    let examples = [
        r#"- Reading notes
--
Chapter 3 was particularly relevant:
 >
  quote from chapter 3
 >
--
"#,
        r#"- Reading notes
--
Chapter 3 was particularly relevant:
>
quote from chapter 3
>
--
"#,
    ];
    assert_yaml_snapshot!(examples.map(parse));
}

#[test]
#[ignore = "parser unimplemented"]
fn tasks() {
    // §2.3 Tasks
    let examples = [r#"- [~] Grocery shopping
-- [x] Eggs
-- [~] Milk
-- [?] Oat or almond?
- [ ] Clean kitchen
- [!] Call dentist
- [-] Return sweater
"#];
    assert_yaml_snapshot!(examples.map(parse));
}

#[test]
fn attributes() {
    // §3 Attributes
    let examples = [r#"##red:My Heading
##red: My Heading

- list item with **red: BOLD** content
"#];
    assert_yaml_snapshot!(examples.map(parse));
}

#[test]
#[ignore = "parser unimplemented"]
fn attribute_chains() {
    // §3.1 Attribute Chains
    let examples = ["##red:underline: My Heading\n"];
    assert_yaml_snapshot!(examples.map(parse));
}

#[test]
#[ignore = "parser unimplemented"]
fn data_attributes() {
    // §3.2 Data Attributes
    let examples = [r#"-{:key "value"}: Table attribute
-["item 1" "item 2"]: List attribute
"#];
    assert_yaml_snapshot!(examples.map(parse));
}
