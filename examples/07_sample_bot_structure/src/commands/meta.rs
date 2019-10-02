use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    FutureCommandResult,
    macros::command,
};

#[command]
async fn ping(ctx: Context, msg: Message) -> FutureCommandResult {
    let _ = msg.channel_id.say(&ctx.http, "Pong!").await;

    (ctx, msg, Ok(()))
}
