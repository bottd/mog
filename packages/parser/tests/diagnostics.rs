use mog_parser::parse_attribute_block;

#[test]
fn diagnostic_lines_use_byte_offsets_after_unicode() {
    for title in ["aaaaaaaa", "😀😀😀😀"] {
        let body = format!("title \"{title}\"\nbad =\nnext 1\nlast 2");
        let error = parse_attribute_block(&body).unwrap_err();

        assert!(error.contains("on line 2 of the block"), "{error}");
    }
}
