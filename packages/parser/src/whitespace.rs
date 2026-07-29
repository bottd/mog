const WHITESPACE: [char; 2] = [' ', '\t'];

pub(crate) fn trim(text: &str) -> &str {
    text.trim_matches(WHITESPACE)
}

pub(crate) fn trim_start(text: &str) -> &str {
    text.trim_start_matches(WHITESPACE)
}

pub(crate) fn trim_end(text: &str) -> &str {
    text.trim_end_matches(WHITESPACE)
}

pub(crate) fn indent(text: &str) -> usize {
    text.len() - trim_start(text).len()
}

pub(crate) fn dedent(text: &str, width: usize) -> &str {
    &text[width.min(indent(text))..]
}
