use serenity::all::{ChannelId, Http};
use tracing::warn;

const CONTINUOUS_STRING_LIMIT: usize = 164;
const PUNCTUATION_SEARCH_START: usize = 144;

#[derive(Debug, PartialEq)]
enum SegmentType {
    Url,
    Base64,
    CodeBlock,
    InlineCode,
    ContinuousString,
    Text,
}

#[derive(Debug)]
struct Segment {
    segment_type: SegmentType,
    content: String,
}

fn is_url_start(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

fn is_url_char(c: char) -> bool {
    !c.is_whitespace()
}

fn is_base64_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
}

fn is_punctuation(c: char) -> bool {
    c == ',' || c == '.' || c == ';' || c == ':' || c == '!' || c == '?'
        || c == '-' || c == ')' || c == ']' || c == '}' || c == '"'
        || c == '\'' || c == '\n'
}

fn parse_segments(content: &str) -> Vec<Segment> {
    let chars: Vec<char> = content.chars().collect();
    let mut segments = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        // Check for fenced code block (```)
        if i + 2 < chars.len() && chars[i] == '`' && chars[i + 1] == '`' && chars[i + 2] == '`' {
            let start = i;
            i += 3;
            while i < chars.len() {
                if i + 2 < chars.len() && chars[i] == '`' && chars[i + 1] == '`' && chars[i + 2] == '`' {
                    i += 3;
                    break;
                }
                i += 1;
            }
            let seg: String = chars[start..i].iter().collect();
            segments.push(Segment { segment_type: SegmentType::CodeBlock, content: seg });
            continue;
        }

        // Check for inline code (`)
        if chars[i] == '`' {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i] != '`' {
                i += 1;
            }
            if i < chars.len() {
                i += 1; // closing backtick
            }
            let seg: String = chars[start..i].iter().collect();
            segments.push(Segment { segment_type: SegmentType::InlineCode, content: seg });
            continue;
        }

        // Check for URL
        let remaining: String = chars[i..].iter().collect();
        if is_url_start(&remaining) {
            let start = i;
            while i < chars.len() && is_url_char(chars[i]) {
                i += 1;
            }
            let seg: String = chars[start..i].iter().collect();
            segments.push(Segment { segment_type: SegmentType::Url, content: seg });
            continue;
        }

        // Check for base64: sequence of base64 chars, reasonably long, no whitespace
        if chars[i].is_ascii_alphanumeric() || chars[i] == '+' || chars[i] == '/' || chars[i] == '=' {
            let start = i;
            let mut b64_count = 0;
            while i < chars.len() && is_base64_char(chars[i]) && !chars[i].is_whitespace() {
                if chars[i].is_ascii_alphanumeric() {
                    b64_count += 1;
                }
                i += 1;
            }
            let seg: String = chars[start..i].iter().collect();
            // Only treat as base64 if it looks like base64: long enough, has mixed case + digits
            if b64_count >= 40 && looks_like_base64(&seg) {
                segments.push(Segment { segment_type: SegmentType::Base64, content: seg });
                continue;
            }
            // Not base64 — fall through to text collection from start position
            i = start;
        }

        // Collect text until next special token
        let start = i;
        while i < chars.len() {
            if chars[i] == '`' {
                break;
            }
            if i + 2 < chars.len() && chars[i] == '`' && chars[i + 1] == '`' && chars[i + 2] == '`' {
                break;
            }
            let remaining_after: String = chars[i..].iter().collect();
            if is_url_start(&remaining_after) {
                break;
            }
            i += 1;
        }
        let seg: String = chars[start..i].iter().collect();
        if !seg.is_empty() {
            if seg.contains(|c: char| c.is_whitespace()) {
                segments.push(Segment { segment_type: SegmentType::Text, content: seg });
            } else {
                segments.push(Segment { segment_type: SegmentType::ContinuousString, content: seg });
            }
        }
    }

    segments
}

fn looks_like_base64(s: &str) -> bool {
    let has_upper = s.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = s.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = s.chars().any(|c| c.is_ascii_digit());
    (has_upper || has_lower) && has_digit
}

fn split_continuous_string(s: &str) -> Vec<String> {
    if s.chars().count() <= CONTINUOUS_STRING_LIMIT {
        return vec![s.to_string()];
    }

    let chars: Vec<char> = s.chars().collect();
    let mut parts = Vec::new();
    let mut start = 0;

    while start < chars.len() {
        let remaining = chars.len() - start;
        if remaining <= CONTINUOUS_STRING_LIMIT {
            let part: String = chars[start..].iter().collect();
            parts.push(part);
            break;
        }

        // Look for punctuation in range 144-164 from start
        let mut split_at = start + CONTINUOUS_STRING_LIMIT;
        let mut found_punct = false;
        let search_end = (start + CONTINUOUS_STRING_LIMIT).min(chars.len());
        let search_start = (start + PUNCTUATION_SEARCH_START).min(chars.len());

        for j in (search_start..search_end).rev() {
            if is_punctuation(chars[j]) {
                split_at = j + 1; // include the punctuation
                found_punct = true;
                break;
            }
        }

        if !found_punct {
            // Hard cut at 164
            split_at = start + CONTINUOUS_STRING_LIMIT;
        }

        let part: String = chars[start..split_at].iter().collect();
        parts.push(part);
        start = split_at;
    }

    parts
}

fn split_text_segment(s: &str, max_chars: usize) -> Vec<String> {
    // First try splitting at newlines
    let lines: Vec<&str> = s.split('\n').collect();
    if lines.len() > 1 {
        let mut parts = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let mut part = line.to_string();
            if i < lines.len() - 1 {
                part.push('\n');
            }
            if !part.is_empty() {
                parts.push(part);
            }
        }
        return parts;
    }

    // No newlines — try splitting at spaces
    let words: Vec<&str> = s.split(' ').collect();
    if words.len() > 1 {
        let mut parts = Vec::new();
        let mut current = String::new();
        for (i, word) in words.iter().enumerate() {
            let candidate = if current.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current, word)
            };
            if candidate.chars().count() > max_chars && !current.is_empty() {
                parts.push(current);
                current = word.to_string();
            } else {
                current = candidate;
            }
            if i == words.len() - 1 && !current.is_empty() {
                parts.push(current.clone());
            }
        }
        return parts;
    }

    // Single word — hard split at char boundary
    let chars: Vec<char> = s.chars().collect();
    let mut parts = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let end = (start + max_chars).min(chars.len());
        let part: String = chars[start..end].iter().collect();
        parts.push(part);
        start = end;
    }
    parts
}

fn assemble_messages(segments: &[Segment], max_chars: usize) -> Vec<String> {
    let mut messages = Vec::new();
    let mut current_msg = String::new();

    for seg in segments {
        let seg_len = seg.content.chars().count();

        // Atomic segments: if adding this would exceed limit, flush current and start new
        if matches!(seg.segment_type, SegmentType::Url | SegmentType::Base64 | SegmentType::CodeBlock | SegmentType::InlineCode) {
            if current_msg.chars().count() + seg_len > max_chars && !current_msg.is_empty() {
                messages.push(current_msg);
                current_msg = String::new();
            }
            // If the atomic segment itself exceeds max, it goes in its own message (whole)
            if seg_len > max_chars {
                if !current_msg.is_empty() {
                    messages.push(current_msg);
                    current_msg = String::new();
                }
                messages.push(seg.content.clone());
                continue;
            }
        }

        // Splittable segments
        match seg.segment_type {
            SegmentType::ContinuousString => {
                let sub_parts = split_continuous_string(&seg.content);
                for sub in sub_parts {
                    // Always flush before a continuous string sub-part to keep them separate
                    if !current_msg.is_empty() {
                        messages.push(current_msg);
                    }
                    // Each continuous string part is its own chunk
                    messages.push(sub);
                    current_msg = String::new();
                }
            }
            SegmentType::Text => {
                let sub_parts = split_text_segment(&seg.content, max_chars);
                for sub in sub_parts {
                    let sub_len = sub.chars().count();
                    if current_msg.chars().count() + sub_len > max_chars && !current_msg.is_empty() {
                        messages.push(current_msg);
                        current_msg = String::new();
                    }
                    current_msg.push_str(&sub);
                }
            }
            _ => {
                // Atomic segments that fit
                if current_msg.chars().count() + seg_len > max_chars && !current_msg.is_empty() {
                    messages.push(current_msg);
                    current_msg = String::new();
                }
                current_msg.push_str(&seg.content);
            }
        }
    }

    if !current_msg.is_empty() {
        messages.push(current_msg);
    }

    messages
}

pub fn split_message(content: &str, max_chars: usize) -> Vec<String> {
    let segments = parse_segments(content);

    // Check if any segment needs splitting
    let needs_splitting = segments.iter().any(|seg| {
        match seg.segment_type {
            SegmentType::ContinuousString => seg.content.chars().count() > CONTINUOUS_STRING_LIMIT,
            SegmentType::Text => seg.content.chars().count() > max_chars,
            _ => false,
        }
    }) || content.chars().count() > max_chars;

    if !needs_splitting {
        return vec![content.to_string()];
    }

    let messages = assemble_messages(&segments, max_chars);

    // Final validation: ensure no message exceeds limit
    let mut result = Vec::new();
    for msg in messages {
        if msg.chars().count() > max_chars && !is_atomic_only(&msg) {
            let chars: Vec<char> = msg.chars().collect();
            let mut start = 0;
            while start < chars.len() {
                let end = (start + max_chars).min(chars.len());
                let part: String = chars[start..end].iter().collect();
                result.push(part);
                start = end;
            }
        } else {
            result.push(msg);
        }
    }

    result
}

fn is_atomic_only(s: &str) -> bool {
    let segments = parse_segments(s);
    segments.iter().all(|seg| {
        matches!(seg.segment_type, SegmentType::Url | SegmentType::Base64 | SegmentType::CodeBlock | SegmentType::InlineCode)
    })
}

pub async fn send_split_message(
    http: &Http,
    channel_id: ChannelId,
    content: &str,
    max_chars: usize,
) {
    let parts = split_message(content, max_chars);

    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let result = channel_id
            .send_message(http, serenity::builder::CreateMessage::new().content(part))
            .await;
        if let Err(e) = result {
            warn!("Failed to send split message part {}/{}: {:?}", i + 1, parts.len(), e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_within_limit_unchanged() {
        let result = split_message("Hello World", 2000);
        assert_eq!(result, vec!["Hello World"]);
    }

    #[test]
    fn test_cjk_within_limit() {
        // CJK text under 164 chars should stay as single segment
        let msg = "你好世界".repeat(40); // 160 chars
        let result = split_message(&msg, 2000);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].chars().count(), 160);
    }

    #[test]
    fn test_cjk_over_continuous_limit_but_under_discord_limit() {
        // CJK text over 164 but under 2000 — split at 164-char boundaries
        let msg = "你好世界".repeat(475); // 1900 chars
        let result = split_message(&msg, 2000);
        assert!(result.len() >= 2);
        let total: usize = result.iter().map(|s| s.chars().count()).sum();
        assert_eq!(total, 1900);
        // Each part should be ≤164 chars (continuous string limit)
        for part in &result {
            assert!(part.chars().count() <= 164);
        }
    }

    #[test]
    fn test_cjk_exceeds_limit() {
        let msg = "你好世界".repeat(525); // 2100 chars
        let result = split_message(&msg, 2000);
        assert!(result.len() >= 2);
        let total: usize = result.iter().map(|s| s.chars().count()).sum();
        assert_eq!(total, 2100);
    }

    #[test]
    fn test_url_preserved_intact() {
        let url = "https://example.com/".to_string() + &"a".repeat(2500);
        let result = split_message(&url, 2000);
        // URL should be whole in one message
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], url);
    }

    #[test]
    fn test_code_block_preserved_intact() {
        let code = "```\n".to_string() + &"fn main() { ".repeat(200) + "\n```";
        let result = split_message(&code, 2000);
        // Code block should be intact
        assert!(result.len() >= 1);
        assert!(result[0].starts_with("```"));
        assert!(result.iter().any(|s| s.ends_with("```")));
    }

    #[test]
    fn test_base64_preserved_intact() {
        let b64 = "SGVsbG8gV29ybGQh".repeat(150); // long base64
        let result = split_message(&b64, 2000);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], b64);
    }

    #[test]
    fn test_continuous_string_under_limit() {
        let s = "a".repeat(150);
        let result = split_message(&s, 2000);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_continuous_string_split_at_punctuation() {
        let mut s = String::new();
        for i in 0..200 {
            if i == 158 {
                s.push(',');
            } else {
                s.push('a');
            }
        }
        let result = split_message(&s, 2000);
        assert!(result.len() >= 2);
        // First part should end at or near the comma
        assert!(result[0].chars().count() >= 155);
        assert!(result[0].chars().count() <= 164);
        assert!(result[0].contains(','));
    }

    #[test]
    fn test_continuous_string_hard_split() {
        let s = "a".repeat(200);
        let result = split_message(&s, 2000);
        assert!(result.len() >= 2);
        assert_eq!(result[0].chars().count(), 164);
    }

    #[test]
    fn test_continuous_string_multiple_splits() {
        let s = "a".repeat(500);
        let result = split_message(&s, 2000);
        assert!(result.len() >= 4); // 500/164 ≈ 4 parts
        let total: usize = result.iter().map(|s| s.chars().count()).sum();
        assert_eq!(total, 500);
    }

    #[test]
    fn test_text_split_at_newlines() {
        let s = "Line1\nLine2\nLine3\n".repeat(150); // long text with newlines
        let result = split_message(&s, 2000);
        assert!(result.len() >= 2);
        for part in &result {
            assert!(part.chars().count() <= 2000);
        }
    }

    #[test]
    fn test_text_split_at_spaces() {
        let s = "word ".repeat(500); // 2500 chars with spaces
        let result = split_message(&s, 2000);
        assert!(result.len() >= 2);
        for part in &result {
            assert!(part.chars().count() <= 2000);
        }
    }

    #[test]
    fn test_no_content_loss() {
        let original = "你好世界".repeat(100) + "\n" + &"Hello ".repeat(200) + "\n" + &"https://example.com/path/to/some/resource/that/is/very/long/and/exceeds/two/thousand/characters/when/combined/with/other/stuff/in/the/message/which/is/important/to/test/comprehensively/including/edge/cases/like/this/very/long/url/that/should/not/be/split".repeat(2);
        let result = split_message(&original, 2000);
        let concatenated: String = result.concat();
        assert_eq!(concatenated, original);
    }

    #[test]
    fn test_mixed_content_split_correctly() {
        let msg = "你好\nhttps://example.com/long/path\n```rust\nfn main() {}\n```\n".repeat(100);
        let result = split_message(&msg, 2000);
        for part in &result {
            assert!(part.chars().count() <= 2000);
        }
    }
}
