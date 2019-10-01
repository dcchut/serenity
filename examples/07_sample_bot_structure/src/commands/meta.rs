use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    FutureCommandResult,
    macros::command,
};
use futures::FutureExt;

#[command]
fn ping(ctx: Context, msg: Message) -> FutureCommandResult {
    async move {
        let _ = msg.channel_id.say(&ctx.http, "Pong!").await;

        (ctx, msg, Ok(()))
    }.boxed()
}
