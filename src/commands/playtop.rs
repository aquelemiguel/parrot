use crate::commands::{play::execute_play, PlayFlag};

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command]
#[aliases("pt")]
async fn playtop(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    execute_play(ctx, msg, args, &PlayFlag::PLAYTOP).await?;
    Ok(())
}
