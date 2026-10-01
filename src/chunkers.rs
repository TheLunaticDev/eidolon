const MAX_CHUNK_CHARS: usize = 1000;

fn ends_sentence(s: &str) -> bool {
    matches!(
        s.trim_end().chars().last(),
        Some('.') | Some('!') | Some('?') | Some('"') | Some('\'')
    )
}

pub fn chunker(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        let line = line.trim();

        if line.is_empty() {
            if !current.is_empty() && ends_sentence(&current) {
                chunks.push(current.trim().to_string());
                current.clear();
            }
            continue;
        }

        for word in line.split_whitespace() {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }

        if current.len() > MAX_CHUNK_CHARS {
            chunks.push(current.trim().to_string());
            current.clear();
        }
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunker_two_paragraphs() {
        let text = "This is first line.\n\nThis is second line.";
        assert_eq!(
            chunker(&text),
            vec![
                "This is first line.",
                "This is second line.",
            ]
        );
    }

    #[test]
    fn test_chunker_multiple_blanks() {
        let text = "This is first line.\n\n\nThis is second line.";
        assert_eq!(
            chunker(&text),
            vec![
                "This is first line.",
                "This is second line.",
            ]
        );
    }

    #[test]
    fn test_chunker_non_ending_chunk_with_multiple_blanks() {
        let text = "This is\n\nfirst line.\n\nThis is \n\n\nsecond line.\n";
        assert_eq!(
            chunker(&text),
            vec![
                "This is first line.",
                "This is second line.",
            ]
        );
    }

    #[test]
    fn test_chunker_removes_carriage_return() {
        let text = "This is a test line.\r\n";
        assert_eq!(chunker(&text), vec!["This is a test line."]);
    }

    #[test]
    fn test_chunker_ignore_whitespace_only_lines() {
        let text = "\t\r\n   \n\r\t";
        assert!(chunker(&text).is_empty());
    }

    #[test]
    fn test_chunker_whitespace_collapse() {
        let text = "This is   \n   a single   line.";
        assert_eq!(
            chunker(&text),
            vec![
                "This is a single line.",
            ]
        );
    }

    #[test]
    fn test_chunker_no_vec_for_empty_input() {
        let text = "";
        assert!(chunker(&text).is_empty());
    }
}
