use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    Args, FutureCommandResult,
    macros::command,
};
use futures::FutureExt;

#[command]
pub fn multiply(ctx: Context, msg: Message, mut args: Args) -> FutureCommandResult {
    async move {
        let one = args.single::<f64>().unwrap();
        let two = args.single::<f64>().unwrap();

        let product = one * two;

        let _ = msg.channel_id.say(&ctx.http, product).await;

        (ctx, msg, Ok(()))
    }.boxed()
}
