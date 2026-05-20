//! Inline-level parsing tests: semantic delimiters within paragraph text.
//!
//! Examples are copied verbatim from spec.mg. See `tests/blocks.rs` for
//! notes on the snapshot workflow.

use insta::assert_yaml_snapshot;
use mog_parser::parse;

#[test]
#[ignore = "parser unimplemented"]
fn escape_sequences() {
    // §1.5 Escape Sequences
    let examples = ["\\**this is not bold\\**\n"];
    assert_yaml_snapshot!(examples.map(parse));
}

#[test]
#[ignore = "parser unimplemented"]
fn strong() {
    // §4 Semantic Delimiters — bold examples
    let examples = [r#"The quick brown fox jumped over the **lazy dog.**

**
The quick brown fox jumped over the lazy dog.
**

A paragraph **with
soft wrapping** in
it.
"#];
    assert_yaml_snapshot!(examples.map(parse));
}

#[test]
#[ignore = "parser unimplemented"]
fn math() {
    // §4.1 Math
    let examples = [r#"Einstein's famous equation $$E = m c^2$$ changed physics.

$$
E^2 = (m c^2)^2 + (p c)^2
$$
"#];
    assert_yaml_snapshot!(examples.map(parse));
}

#[test]
#[ignore = "parser unimplemented"]
fn verbatim() {
    // §4.2 Verbatim
    let examples = [
        r#"Some code with ``python: print("Hello, world!")``

``python:
def greet(name):
    print(f"Hello, {name}!")
``
"#,
        r#"-
  A list item
  ``python:
  def greet(name):
    print(f"Hello, {name}!")
  ``
-
"#,
    ];
    assert_yaml_snapshot!(examples.map(parse));
}

#[test]
#[ignore = "parser unimplemented"]
fn tables() {
    // §4.3 Tables
    let examples = [
        r#"#| Name       || Type      || Color    ||
-| Apple      || Fruit     || Red      ||
-| **Carrot** || Vegetable || Orange   ||
-| Blueberry  || Fruit     || Blue     ||
"#,
        "#| Name || Type || Color || -| Banana || Fruit || Yellow ||\n",
        r#"#| Name       || Type      || Color    ||
-| Apple      || Fruit     ||
-| Carrot     || Vegetable || Orange   || Crunchy ||
-| Blueberry  || Fruit     || Blue     ||
"#,
        r#"-| Apple      || Fruit     ||
-| Carrot     || Vegetable || Orange   || Crunchy ||
-| Blueberry  || Fruit     || Blue     ||
"#,
        r#"#|            ||           ||          || Texture ||
#| Name       || Type      || Color    ||
-| Apple      || Fruit     ||
-| Blueberry  || Fruit     || Blue     ||
#| Vegetables ||
-| Carrot     || Vegetable || Orange   || Crunchy ||
"#,
    ];
    assert_yaml_snapshot!(examples.map(parse));
}

#[test]
#[ignore = "parser unimplemented"]
fn links() {
    // §4.4 Links
    let examples = [
        r#"- [[https://kdl.dev]]
- [[https://kdl.dev]]((KDL))
- [[https://kdl.dev]]((KDL)){{ A node-based document language }}
"#,
        r#"[[recipe]]               Document reference
[[#:Ingredients]]        Heading in current document
[[https://kdl.dev]]      External via protocol attribute
"#,
        r#"[[!:recipe]]       Document Transclusion
[[!:image.png]]    Image Transclusion
"#,
    ];
    assert_yaml_snapshot!(examples.map(parse));
}

#[test]
#[ignore = "parser unimplemented"]
fn footnotes() {
    // §4.5 Footnotes
    let examples = [
        r#"A good marinara starts with San Marzano
[[tomato]]((tomatoes)) and a generous amount
of olive oil.

[[tomato]]{{ A fruit not a vegetable }}
"#,
        r#"A good marinara starts with San Marzano
tomato{{ A tomato is a fruit not a vegetable }} and a generous amount
of olive oil.
"#,
    ];
    assert_yaml_snapshot!(examples.map(parse));
}
