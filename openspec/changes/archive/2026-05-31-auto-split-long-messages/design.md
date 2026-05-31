## Context

The opencb CLI sends messages to Discord via two paths:
- **Send command** (`main.rs`): User runs `opencb send "message"` — passes content directly to Discord API with no length check
- **Serve mode** (`handler.rs`): Bot replies to mentions — truncates content at 1900 bytes using `truncate()` function

Discord's API limit is 2000 characters per message. The current `truncate()` function has critical flaws:
1. Uses byte length (`.len()`) instead of character count (`.chars().count()`) — CJK/Japanese content is 3 bytes per char, so only ~667 chars get through
2. Silently drops content — no multi-message delivery
3. Doesn't account for Discord's special formatting (URLs auto-unfold, mentions expand, code blocks must be balanced)

## Goals / Non-Goals

**Goals:**
- Deliver complete message content regardless of length — split into multiple messages
- Correct character counting using Unicode codepoints
- Preserve readability: never split mid-URL, mid-code-block, mid-inline-code, or mid-word for Latin text
- Handle multi-language content: CJK, Japanese, mixed scripts
- Apply to both send command and serve mode reply paths
- Rate-limit-aware sequential sending

**Non-Goals:**
- Embed/rich message formatting
- Message priority or ordering guarantees beyond sequential
- Streaming partial messages in real-time
- Supporting Discord's 6000-char webhook limit (stick to 2000 for bot messages)

## Decisions

### Decision: Content parsing into atomic vs splittable segments

**Chosen approach**: Parse message content into typed segments with two categories:

**Atomic segments** (never split internally, sent whole in one message):
- URLs: `http://` or `https://` prefix, terminated by whitespace or end-of-string
- Base64 data: continuous alphanumeric + `+/=` characters, no spaces/newlines
- Fenced code blocks: triple backtick pairs (```...```)
- Inline code: single backtick pairs (`...`)

**Splittable segments** (can be split at internal boundaries):
- Continuous string: no spaces or newlines — split at 164 chars, preferring punctuation within last 20 chars (range 144-164)
- Normal text: contains spaces or newlines — split at `\n` then space boundaries

**Rationale**: URLs and code blocks are semantic units — splitting them produces broken output. Base64 is not human-readable, so splitting serves no purpose. Continuous strings (e.g., long identifiers, hashtags) benefit from a shorter split threshold (164) to keep messages scannable.

**Alternative considered**: Regex-based splitting — rejected because regex struggles with nested patterns and multi-line code blocks. Simple scanner is more predictable.

### Decision: Split strategy for continuous strings

**Chosen approach**: 164-character soft limit with punctuation preference:
1. If continuous string ≤164 chars: keep as single segment
2. If >164 chars: scan positions 144-164 for punctuation/dash boundary
3. If punctuation found: split there
4. If no punctuation: hard split at 164
5. Repeat for remaining text

**Rationale**: 164 is a readability threshold — long unbroken strings are hard to scan in Discord. Preferring punctuation boundaries produces more natural breaks. The 20-char search window (144-164) balances readability with not going too far from the target.

### Decision: Split strategy cascade for text segments

**Chosen approach**: Two-tier fallback for text with spaces/newlines:
1. Split at `\n` boundaries (best readability)
2. If a single line exceeds max, split at space boundaries

**Rationale**: Most Discord messages have newlines or spaces. CJK text without spaces is rare in long messages.

### Decision: Character counting

**Chosen approach**: Use `.chars().count()` for limit comparison. Use `.len()` only for byte-level operations (slicing).

**Rationale**: Discord counts Unicode scalar values. Rust's `.chars()` iterator yields Unicode scalar values. This matches Discord's counting for most content. (Edge case: ZWJ emoji sequences like 👨‍👩‍👧‍👦 are multiple scalar values but display as one — acceptable imprecision.)

### Decision: Shared utility module

**Chosen approach**: Create `src/splitter.rs` with `pub fn split_message(content: &str, max_chars: usize) -> Vec<String>`. Import from both `main.rs` and `handler.rs`.

**Rationale**: Single implementation, single test surface, consistent behavior across all send paths.

### Decision: Rate limiting between split messages

**Chosen approach**: Add a 100ms `tokio::time::sleep` between sequential sends in the split loop.

**Rationale**: Discord rate limit is ~5 messages/second per channel. 100ms delay = 10 msg/sec headroom. Prevents 429 errors without noticeable delay for the user. The delay is applied in the send loop, not in the splitter function (separation of concerns).

## Risks / Trade-offs

- **[Risk] ZWJ emoji sequences counted incorrectly** → Mitigation: Acceptable imprecision; Discord's own counting is inconsistent across clients. Worst case: one extra split.
- **[Risk] Very long single-token content (base64 dump) produces many tiny messages** → Mitigation: Log a warning when splitting produces >10 messages. User can pipe through `fold` or similar.
- **[Risk] Rate limit hit if splitting produces many messages quickly** → Mitigation: 100ms delay between sends. If 429 received, retry with exponential backoff (serenity handles this).
- **[Trade-off] Code block detection is heuristic, not full markdown parser** → Acceptable because the vast majority of code blocks use standard triple-backtick syntax.
