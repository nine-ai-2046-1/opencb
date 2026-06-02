//! 📢 Echo slash command
//! Returns the args string exactly as received, preserving all formatting ✨

use super::{CommandContext, SlashCommand};

pub struct EchoCommand;

impl SlashCommand for EchoCommand {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes back the input message"
    }

    fn execute(&self, ctx: &CommandContext) -> String {
        ctx.args.to_string()
    }
}
