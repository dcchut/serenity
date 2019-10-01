use crate::ShardManagerContainer;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    FutureCommandResult,
    macros::command,
};
use futures::FutureExt;

#[command]
#[owners_only]
fn quit(ctx: Context, msg: Message) -> FutureCommandResult {
    async move {
        {
            let data = ctx.data.read().await;

            if let Some(manager) = data.get::<ShardManagerContainer>() {
                manager.lock().await.shutdown_all();
                let _ = msg.reply(&ctx, "Shutting down!").await;

            } else {
                let _ = msg.reply(&ctx, "There was a problem getting the shard manager").await;
            }
        }
        (ctx, msg, Ok(()))
    }.boxed()
}
