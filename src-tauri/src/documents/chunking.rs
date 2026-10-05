//! Splits text into fragments for search (paragraph-aware, bounded size).

const MAX_CHARS: usize = 1000;

pub fn chunk_text(text: &str) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut flush = |cur: &mut String, chunks: &mut Vec<String>| {
        if !cur.trim().is_empty() {
            chunks.push(cur.trim().to_string());
        }
        cur.clear();
    };
    for para in text.split("\n\n").map(str::trim).filter(|p| !p.is_empty()) {
        if para.chars().count() > MAX_CHARS {
            flush(&mut cur, &mut chunks);
            // very long paragraph: cut at sentence-ish boundaries by words
            let mut piece = String::new();
            for word in para.split_whitespace() {
                if piece.chars().count() + word.chars().count() + 1 > MAX_CHARS {
                    chunks.push(piece.trim().to_string());
                    piece.clear();
                }
                piece.push_str(word);
                piece.push(' ');
            }
            flush(&mut piece, &mut chunks);
            continue;
        }
        if cur.chars().count() + para.chars().count() + 2 > MAX_CHARS {
            flush(&mut cur, &mut chunks);
        }
        if !cur.is_empty() {
            cur.push_str("\n\n");
        }
        cur.push_str(para);
    }
    flush(&mut cur, &mut chunks);
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_is_one_chunk() {
        assert_eq!(chunk_text("Uno.\n\nDos."), vec!["Uno.\n\nDos."]);
    }

    #[test]
    fn empty_text_has_no_chunks() {
        assert!(chunk_text("  \n\n ").is_empty());
    }

    #[test]
    fn long_text_is_split_and_nothing_is_lost() {
        let para = "palabra ".repeat(100); // 800 chars
        let text = format!("{para}\n\n{para}\n\n{para}");
        let chunks = chunk_text(&text);
        assert_eq!(chunks.len(), 3);
        assert!(chunks.iter().all(|c| c.chars().count() <= 1000));
        let giant = "x ".repeat(2500);
        let c = chunk_text(&giant);
        assert!(c.len() >= 5 && c.iter().all(|c| c.chars().count() <= 1000));
        assert_eq!(c.join(" ").split_whitespace().count(), 2500);
    }
}
