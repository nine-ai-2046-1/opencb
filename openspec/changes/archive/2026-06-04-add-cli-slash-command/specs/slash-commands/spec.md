## MODIFIED Requirements

### Requirement: SlashCommand trait definition
The system SHALL define a `SlashCommand` trait in `src/slash_commands/mod.rs` with:
- `fn name(&self) -> &str` (sync, unchanged)
- `fn description(&self) -> &str` (sync, unchanged)
- `fn options(&self) -> Vec<CreateCommandOption>` (sync, default `vec![]`, unchanged)
- `async fn execute(&self, ctx: &CommandContext) -> String` — upgraded from sync to async; all commands MUST implement this
- `async fn execute_with_updates(&self, ctx: &CommandContext, handle: &ResponseHandle)` — default implementation calls `self.execute(ctx).await` and then `handle.finalize(&result).await`; streaming commands MAY override this method

#### Scenario: Simple command uses default execute_with_updates
- **WHEN** a command implements only `async fn execute()` and `execute_with_updates()` is called
- **THEN** the default implementation SHALL call `execute()` once and pass the result to `handle.finalize()`

#### Scenario: Streaming command overrides execute_with_updates
- **WHEN** a command overrides `execute_with_updates()` and the handler calls it
- **THEN** the command MAY call `handle.update()` multiple times before calling `handle.finalize()`

## ADDED Requirements

### Requirement: ResponseHandle for interaction updates
The system SHALL provide a `ResponseHandle` struct in `src/slash_commands/mod.rs` that wraps `Arc<serenity::all::Http>`, an `ApplicationId`, and an interaction token string. It SHALL expose:
- `async fn update(&self, content: &str)` — edits the deferred interaction response with new content
- `async fn finalize(&self, content: &str)` — makes the final edit to the interaction response

#### Scenario: Intermediate update via handle
- **WHEN** `handle.update("step 1 done")` is called
- **THEN** the Discord interaction message SHALL be edited to show `"step 1 done"`

#### Scenario: Final update via handle
- **WHEN** `handle.finalize("all done")` is called
- **THEN** the Discord interaction message SHALL be edited to show `"all done"` as the final response
