//! 📢 Echo slash command
//! Returns the args string exactly as received, preserving all formatting ✨

use serenity::all::CommandOptionType;
use serenity::builder::CreateCommandOption;

use super::{CommandContext, SlashCommand};

pub struct EchoCommand;

impl SlashCommand for EchoCommand {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes back the input message"
    }

    /// Declares one required String option named "text".
    /// Discord will show a text input field when the user types /echo.
    fn options(&self) -> Vec<CreateCommandOption> {
        vec![
            CreateCommandOption::new(CommandOptionType::String, "text", "Message to echo back")
                .required(true),
        ]
    }

    /// Returns ctx.args verbatim (the value of the "text" option).
    async fn execute(&self, ctx: &CommandContext) -> String {
        ctx.args.to_string()
    }
}
