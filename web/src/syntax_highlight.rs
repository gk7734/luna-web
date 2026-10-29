use anyhow::{Context as _, Result};
use gpui::{HighlightStyle, rgb};
use std::ops::Range;
use streaming_iterator::StreamingIterator as _;
use tree_sitter::{Parser, Query, QueryCursor};

const RUST_HIGHLIGHTS: &str = include_str!("../../crates/grammars/src/rust/highlights.scm");

/// Highlights `source` with Luna's own Rust highlight query, returning sorted, non-overlapping
/// ranges as `StyledText` requires.
pub fn highlight_rust(source: &str) -> Result<Vec<(Range<usize>, HighlightStyle)>> {
    let language: tree_sitter::Language = tree_sitter_rust::LANGUAGE.into();
    let mut parser = Parser::new();
    parser.set_language(&language)?;
    let tree = parser
        .parse(source, None)
        .context("tree-sitter returned no tree")?;
    let query = Query::new(&language, RUST_HIGHLIGHTS)?;

    // Captures overlap, e.g. a function name inside a call; the first capture of a byte wins, as
    // in Tree-sitter's own highlighter.
    let mut byte_colors: Vec<Option<u32>> = vec![None; source.len()];
    let mut cursor = QueryCursor::new();
    let mut captures = cursor.captures(&query, tree.root_node(), source.as_bytes());
    while let Some((query_match, capture_index)) = captures.next() {
        let Some(capture) = query_match.captures.get(*capture_index) else {
            continue;
        };
        let Some(capture_name) = query.capture_names().get(capture.index as usize) else {
            continue;
        };
        let Some(color) = color_for_capture(capture_name) else {
            continue;
        };
        for byte_color in byte_colors
            .get_mut(capture.node.byte_range())
            .into_iter()
            .flatten()
        {
            byte_color.get_or_insert(color);
        }
    }

    let mut highlights = Vec::new();
    let mut run_start = 0;
    for offset in 1..=source.len() {
        let run_ends = offset == source.len()
            || byte_colors.get(offset) != byte_colors.get(run_start)
                && source.is_char_boundary(offset);
        if run_ends {
            if let Some(Some(color)) = byte_colors.get(run_start) {
                highlights.push((run_start..offset, rgb(*color).into()));
            }
            run_start = offset;
        }
    }
    Ok(highlights)
}

fn color_for_capture(capture_name: &str) -> Option<u32> {
    let category = capture_name.split('.').next()?;
    Some(match category {
        "keyword" => 0xc678dd,
        "function" => 0x61afef,
        "type" | "constructor" => 0xe5c07b,
        "string" => 0x98c379,
        "comment" => 0x7f848e,
        "number" | "boolean" | "constant" => 0xd19a66,
        "attribute" | "lifetime" => 0x56b6c2,
        "property" => 0xe06c75,
        "variable" if capture_name.ends_with("special") => 0xe06c75,
        "operator" | "punctuation" => 0xabb2bf,
        _ => return None,
    })
}
