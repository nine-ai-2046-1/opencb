## Context

OpenCB is a Discord Bot that bridges CLI tools with Discord channels. Currently, when users send messages with line breaks or markdown formatting to Discord, the bot processes them through `handler.rs` which aggressively normalizes whitespace (lines 64-69), collapsing all newlines and multiple spaces into single spaces. Additionally, CLI output sent back to Discord may not preserve markdown formatting.

The current code path:
1. User sends message with line breaks/markdown to Discord
2. Bot receives in `handler.rs`
3. Whitespace normalization collapses line breaks
4. CLI processes the flattened input
5. CLI output is sent back via `outbound.rs`

## Goals / Non-Goals

**Goals:**
- Preserve line breaks in incoming Discord messages for CLI processing
- Ensure markdown formatting in CLI output renders properly in Discord
- Maintain backward compatibility with existing functionality

**Non-Goals:**
- Parsing or validating markdown syntax
- Adding new markdown features beyond what Discord natively supports
- Changing the CLI execution behavior

## Decisions

### 1. Modify whitespace normalization to preserve line breaks

**Current approach:** `split_whitespace().join(" ")` collapses all whitespace to single spaces.

**New approach:** Keep single space normalization but preserve line breaks (`\n`).

**Rationale:** Line breaks are meaningful for multi-line input to CLIs. Multiple spaces are rarely significant, so collapsing them is acceptable. Discord's markdown uses `\n` for line breaks.

### 2. Wrap CLI output in Discord code blocks when contains markdown

**Decision:** Add optional flag or auto-detect when to wrap output in ```code blocks.

**Rationale:** Some CLI outputs contain markdown that Discord may misinterpret. Wrapping in code blocks ensures literal display when needed. However, this may not be desired for all outputs, so making it optional or context-aware is better.

**Alternative considered:** Always wrap in code blocks - rejected as it would break intentional markdown rendering.

## Risks / Trade-offs

- **[Risk]** Some CLIs may rely on collapsed whitespace → **Mitigation:** Document the change; users can adjust CLI behavior if needed
- **[Risk]** Markdown auto-detection may cause unexpected formatting → **Mitigation:** Default to passing through; add opt-in for code block wrapping
- **[Trade-off]** Preserving all whitespace may increase token count for CLI input → **Mitigation:** Acceptable trade-off for correct formatting
