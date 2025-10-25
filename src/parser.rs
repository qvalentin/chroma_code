use regex::Regex;
use scraper::{Html, Selector};

use crate::{HighlightedText, CliArgs};

#[derive(Debug)]
pub struct HeaderInfo {
    pub caption: Option<String>,
    pub label: Option<String>,
}

/// Parses the first line of the input file for a header comment, which can be used to set the
/// caption and label for the listing. The format is `comment_char chroma_code: caption: Your Caption label: your-label`.
/// Both caption and label are optional. The comment characters are passed via `comment_types`.
pub fn parse_header(
    content: &str,
    comment_types: &[&str],
) -> Option<HeaderInfo> {
    if let Some(first_line) = content.lines().next() {
        for comment_type in comment_types {
            if first_line.starts_with(comment_type) {
                let trimmed_line = first_line[comment_type.len()..].trim();
                if let Some(captures) = Regex::new(
                    r"chroma_code:\s*(?:caption:\s*(?P<caption>.*?))?\s*(?:label:\s*(?P<label>.*?))?$",
                )
                .unwrap()
                .captures(trimmed_line)
                {
                    let caption = captures.name("caption").map(|m| m.as_str().trim().to_string());
                    let label = captures.name("label").map(|m| m.as_str().trim().to_string());

                    // If either caption or label is found, we consider it a match
                    if caption.is_some() || label.is_some() {
                        let header_info = HeaderInfo { caption, label };
                            return Some(header_info)
                    }
                }
            }
        }
    }
    None
}

/// Parses the html string and returns a vector of `HighlightedText`
fn parse(html_string: &str, conf: &CliArgs) -> Vec<HighlightedText> {
    let document = Html::parse_document(html_string);
    let selector = Selector::parse("pre > code > span").unwrap();
    let mut highlighted_text_pieces: Vec<HighlightedText> = vec![];
    for element in document.select(&selector) {
        let style = element.value().attr("style").unwrap_or("");
        let hex_color = Regex::new(r"color:\s*#([0-9a-fA-F]{6});")
            .unwrap()
            .captures(style)
            .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .unwrap_or_else(|| conf.default_color.clone());

        let bold = style.contains("font-weight: bold");
        let italic = style.contains("font-style: italic");
        let underline = style.contains("text-decoration: underline");

        let text = element.text().collect::<String>();

        highlighted_text_pieces.push(HighlightedText {
            text,
            hex_color,
            bold,
            italic,
            underline,
        });
    }
    if conf.verbose {
        println!(
            "Successfully parsed the html string, found {} highlighted text pieces.",
            highlighted_text_pieces.len()
        );
    }
    highlighted_text_pieces
}

pub fn extract_highlighted_pieces(html_bytes: Vec<u8>, conf: &CliArgs) -> Vec<HighlightedText> {
    let html_string = String::from_utf8(html_bytes).unwrap();
    parse(&html_string, conf)
}
