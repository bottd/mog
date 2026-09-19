use insta::glob;
use mog_parser::{Attributes, Document, MarkerKind, Node, NodeKind, Value, parse, parse_unfolded};

fn walk<'a>(nodes: &'a [Node], out: &mut Vec<&'a Node>) {
    for node in nodes {
        out.push(node);
        walk(&node.children, out);
    }
}

fn nodes(document: &Document) -> Vec<&Node> {
    let mut out = Vec::new();
    walk(&document.body, &mut out);
    out
}

fn line_of(src: &str, offset: usize) -> usize {
    src[..offset].matches('\n').count()
}

// `impact 1` is a KDL node whose sole argument carries the value.
fn impact(attributes: &Attributes) -> &Value {
    let value = &attributes
        .children
        .iter()
        .find(|attribute| attribute.name.as_deref() == Some("impact"))
        .expect("impact attribute")
        .value;

    match value {
        Value::Node(node) => &node.entries.first().expect("argument").value,
        other => other,
    }
}

fn assert_spans_describe(src: &str) {
    for document in [parse(src), parse_unfolded(src)] {
        for node in nodes(&document) {
            let Some(span) = node.span else { continue };

            assert!(span.start <= span.end, "inverted span {span:?}");
            assert!(span.end <= src.len(), "span past end {span:?}");
            let text = &src[span.start..span.end];

            match &node.kind {
                NodeKind::Marker(marker) => assert_eq!(
                    MarkerKind::from_byte(text.as_bytes()[0]),
                    Some(marker.kind),
                    "marker span does not open with its own marker: {text:?}"
                ),
                NodeKind::Attributes => assert!(
                    text.starts_with("``"),
                    "attribute span does not open with a fence: {text:?}"
                ),
                _ => {}
            }

            assert!(
                !text.ends_with('\n') && !text.ends_with('\r'),
                "span swallows its line ending: {text:?}"
            );
            if let Some(next) = src[span.end..].chars().next() {
                assert!(
                    next == '\n' || next == '\r',
                    "span stops mid-line before {next:?}"
                );
            }

            assert_eq!(
                span.start_line,
                line_of(src, span.start),
                "wrong start_line"
            );
            assert_eq!(span.end_line, line_of(src, span.end), "wrong end_line");
        }
    }
}

#[test]
fn every_span_covers_its_own_syntax_through_the_end_of_its_last_line() {
    glob!("fixtures", "**/*.mg", |path| {
        assert_spans_describe(&std::fs::read_to_string(path).unwrap());
    });
}

#[test]
fn the_spec_document_is_described_by_its_spans() {
    assert_spans_describe(&std::fs::read_to_string("../../spec.mg").unwrap());
}

#[test]
fn an_attr_block_span_covers_both_of_its_fences() {
    let src = "=hero:\n``attr:\nimpact 1\n``\n## Abrams\n=\n";
    let document = parse_unfolded(src);
    let block = nodes(&document)
        .into_iter()
        .find(|node| node.kind == NodeKind::Attributes)
        .expect("attribute node");
    let span = block.span.expect("block span");

    assert_eq!(&src[span.start..span.end], "``attr:\nimpact 1\n``");
}

#[test]
fn replacing_a_block_through_its_span_only_changes_its_attributes() {
    let src = "=hero:\n``attr:\nimpact 1\n``\n## Abrams\n=\n";
    let document = parse(src);
    let marker = &document.body[0];
    let span = marker.attributes.as_ref().unwrap().blocks[0];

    let rewritten = format!(
        "{}``attr:\nimpact 2\n``{}",
        &src[..span.start],
        &src[span.end..]
    );
    let after = parse(&rewritten);

    assert_eq!(
        impact(after.body[0].attributes.as_ref().unwrap()),
        &Value::Int(2)
    );
    assert_eq!(marker.children, after.body[0].children);
}

#[test]
fn inserting_after_a_fence_line_attaches_the_block_to_that_marker() {
    let src = "=hero:\n## Abrams\n=\n";
    let document = parse(src);
    let fence = document.body[0].fence.expect("marker fence");
    assert_eq!(&src[fence.start..fence.end], "=hero:");

    let rewritten = format!(
        "{}\n``attr:\nimpact 2\n``{}",
        &src[..fence.end],
        &src[fence.end..]
    );
    let after = parse(&rewritten);

    assert_eq!(
        impact(after.body[0].attributes.as_ref().unwrap()),
        &Value::Int(2)
    );
}

#[test]
fn an_inline_attr_block_contributes_no_spliceable_span() {
    let block = parse("=hero:\n``attr:\nk 1\n``\n=\n");
    assert_eq!(block.body[0].attributes.as_ref().unwrap().blocks.len(), 1);

    let inline = parse("# Heading ``attr: k 1``\n");
    let attributes = inline.body[0].attributes.as_ref().unwrap();

    assert!(
        !attributes.children.is_empty(),
        "inline attributes still fold"
    );
    assert!(attributes.blocks.is_empty(), "{:?}", attributes.blocks);
}

#[test]
fn an_empty_attr_block_still_records_where_it_is() {
    let document = parse("=hero:\n``attr:\n``\n=\n");
    let attributes = document.body[0].attributes.as_ref().unwrap();

    assert!(attributes.children.is_empty());
    assert_eq!(attributes.blocks.len(), 1);
}

#[test]
fn several_root_blocks_record_every_span_in_source_order() {
    let src = "``attr:\ntitle \"First\"\n``\n\n# Heading\n\n``attr:\nversion 2\n``\n";
    let document = parse(src);
    let blocks = &document.attributes.as_ref().unwrap().blocks;

    assert_eq!(blocks.len(), 2);
    assert_eq!(
        &src[blocks[0].start..blocks[0].end],
        "``attr:\ntitle \"First\"\n``"
    );
    assert_eq!(
        &src[blocks[1].start..blocks[1].end],
        "``attr:\nversion 2\n``"
    );
}

#[test]
fn crlf_spans_are_byte_correct_for_the_string_as_given() {
    let src = "=hero:\r\n``attr:\r\nimpact 1\r\n``\r\n=\r\n";
    let span = parse(src).body[0].attributes.as_ref().unwrap().blocks[0];

    assert_eq!(&src[span.start..span.end], "``attr:\r\nimpact 1\r\n``");
    assert_eq!((span.start_line, span.end_line), (1, 3));
}

#[test]
fn a_bom_does_not_shift_spans_off_the_original_string() {
    let src = "\u{FEFF}# Heading\n";
    let span = parse(src).body[0].span.unwrap();

    assert_eq!(&src[span.start..span.end], "# Heading");
    assert_eq!(span.start, "\u{FEFF}".len());
}

#[test]
fn an_indented_span_starts_at_the_marker_not_the_indent() {
    let src = "=block:\n    # Heading\n=\n";
    let document = parse(src);
    let span = document.body[0].children[0].span.unwrap();

    assert_eq!(&src[span.start..span.end], "# Heading");
}

#[test]
fn a_block_marker_spans_both_fences_and_keeps_the_opening_one() {
    let src = "=hero:\n# A\n# B\n=\n";
    let node = &parse(src).body[0];
    let span = node.span.unwrap();
    let fence = node.fence.unwrap();

    assert_eq!(&src[span.start..span.end], src.trim_end());
    assert_eq!(&src[fence.start..fence.end], "=hero:");
}

#[test]
fn a_single_line_marker_is_its_own_fence_and_does_not_repeat_it() {
    assert!(parse("# Heading\n").body[0].fence.is_none());
}

#[test]
fn an_attached_attribute_block_preserves_the_markers_opening_line() {
    for opening in ["# Heading", "- item", ". item", "> quote", "= free"] {
        let src = format!("{opening}\n``attr:\nk 1\n``\n``attr:\nj 2\n``\n");
        for document in [parse(&src), parse_unfolded(&src)] {
            let marker = &document.body[0];
            let span = marker.span.unwrap();
            let fence = marker.fence.expect("opening line survives span growth");

            assert_eq!(&src[fence.start..fence.end], opening);
            assert_eq!(&src[span.start..span.end], src.trim_end());

            // Editing just the opening line must leave both blocks intact.
            let rewritten = format!("# Renamed{}", &src[fence.end..]);
            assert_eq!(rewritten, "# Renamed\n``attr:\nk 1\n``\n``attr:\nj 2\n``\n");
            let after = parse(&rewritten);
            let before = parse(&src);
            assert_eq!(
                after.body[0].attributes.as_ref().unwrap().children,
                before.body[0].attributes.as_ref().unwrap().children
            );
        }
    }
}

#[test]
fn an_unclosed_block_ends_at_the_last_line() {
    let src = "=hero:\n# A\n";
    let span = parse(src).body[0].span.unwrap();

    assert_eq!(&src[span.start..span.end], "=hero:\n# A");
    assert_eq!(span.end_line, 1);
}

#[test]
fn an_unclosed_block_ends_at_the_last_line_with_something_on_it() {
    for (src, expected) in [
        ("=hero:\n# A\n\n", "=hero:\n# A"),
        ("=hero:\n\n\n", "=hero:"),
        ("``\nfoo\n\n", "``\nfoo"),
        ("``attr:\nk 1\n\n", "``attr:\nk 1"),
    ] {
        let span = parse(src).body[0].span.unwrap();
        assert_eq!(&src[span.start..span.end], expected, "for {src:?}");
    }
}

#[test]
fn a_bare_trailing_carriage_return_is_content_not_a_line_ending() {
    let src = "# A\r";
    let span = parse(src).body[0].span.unwrap();

    assert_eq!(&src[span.start..span.end], "# A\r");
}

#[test]
fn an_owner_covers_the_attr_block_attached_to_it() {
    for src in [
        "# H\n``attr:\nk 1\n``\n",
        "-| a\n``attr:\nk 1\n``\n",
        "para\n``attr:\nk 1\n``\n",
        "- item\n``attr:\nk 1\n``\n",
    ] {
        let document = parse_unfolded(src);
        let owner = &document.body[0];
        let span = owner.span.expect("owner span");
        let block = owner
            .children
            .iter()
            .find_map(|child| {
                (child.kind == NodeKind::Attributes)
                    .then_some(child.span)
                    .flatten()
            })
            .expect("attribute block");

        assert!(
            span.start <= block.start && block.end <= span.end,
            "owner {span:?} does not cover its block {block:?} for {src:?}"
        );
    }
}

#[test]
fn a_single_line_marker_that_grows_over_a_block_records_its_own_line() {
    let src = "# Title\n``attr:\nk 1\n``\n";
    let marker = &parse(src).body[0];
    let span = marker.span.unwrap();
    let fence = marker
        .fence
        .expect("fence once the span is more than one line");

    assert_eq!(&src[span.start..span.end], src.trim_end());
    assert_eq!(&src[fence.start..fence.end], "# Title");
}

#[test]
fn a_fence_is_present_exactly_when_the_span_is_more_than_its_first_line() {
    for src in [
        "# Title\n",
        "- item\n",
        "# Title\n``attr:\nk 1\n``\n",
        "=hero:\n# A\n=\n",
        "- item\n``attr:\nk 1\n``\n",
    ] {
        let node = &parse(src).body[0];
        let span = node.span.unwrap();
        let grew = span.end_line > span.start_line;

        assert_eq!(node.fence.is_some(), grew, "for {src:?}");
    }
}
