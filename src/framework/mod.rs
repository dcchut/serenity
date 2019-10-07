//! The framework is a customizable method of separating commands.
//!
//! This is used in combination with [`Client::with_framework`].
//!
//! The framework has a number of configurations, and can have any number of
//! commands bound to it. The primary purpose of it is to offer the utility of
//! not needing to manually match message content strings to determine if a
//! message is a command.
//!
//! Additionally, "checks" can be added to commands, to ensure that a certain
//! condition is met prior to calling a command; this could be a check that the
//! user who posted a message owns the bot, for example.
//!
//! Each command has a given name, and an associated function/closure. For
//! example, you might have two commands: `"ping"` and `"weather"`. These each
//! have an associated function that are called if the framework determines
//! that a message is of that command.
//!
//! Assuming a command prefix of `"~"`, then the following would occur with the
//! two previous commands:
//!
//! ```ignore
//! ~ping // calls the ping command's function
//! ~pin // does not
//! ~ ping // _does_ call it _if_ the `allow_whitespace` option is enabled
//! ~~ping // does not
//! ```
//!
//! # Examples
//!
//! Configuring a Client with a framework, which has a prefix of `"~"` and a
//! ping and about command:
//!
//! ```rust,no_run
//! #![feature(async_closure)]
//! use serenity::client::{Client, Context, EventHandler};
//! use serenity::model::channel::Message;
//! use serenity::framework::standard::macros::{command, group};
//! use serenity::framework::standard::{StandardFramework, CommandResult};
//! use std::env;
//!
//! #[command]
//! async fn about(ctx: &mut Context, msg: &Message) -> CommandResult {
//!     msg.channel_id.say(&ctx.http, "A simple test bot").await?;
//!
//!     Ok(())
//! }
//!
//! #[command]
//! async fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
//!     msg.channel_id.say(&ctx.http, "pong!").await?;
//!
//!     Ok(())
//! }
//!
//! #[group]
//! #[commands(about, ping)]
//! struct General;
//!
//! struct Handler;
//!
//! impl EventHandler for Handler {}
//!
//! # #[tokio::main]
//! # async fn main() {
//! let token = env::var("DISCORD_TOKEN").unwrap();
//! let mut client = Client::new(&token, Handler).await.unwrap();
//!
//! client.with_framework(StandardFramework::new()
//!     .configure(|c| c.prefix("~"))
//!     // The `#[group]` (and similarly, `#[command]`) macro generates static instances
//!     // containing any options you gave it. For instance, the group `name` and its `commands`.
//!     // Their identifiers, names you can use to refer to these instances in code, are an
//!     // all-uppercased version of the `name` with a `_GROUP` suffix appended at the end.
//!     .group(&GENERAL_GROUP)).await;
//! # }
//! ```
//!
//! [`Client::with_framework`]: ../client/struct.Client.html#method.with_framework

#[cfg(feature = "standard_framework")]
pub mod standard;

#[cfg(feature = "standard_framework")]
pub use self::standard::StandardFramework;

use crate::client::Context;
use crate::model::channel::Message;
use async_trait::async_trait;
use std::sync::Arc;

/// A trait for defining your own framework for serenity to use.
///
/// Should you implement this trait, or define a `message` handler, depends on you.
/// However, using this will benefit you by abstracting the `EventHandler` away,
/// and providing a reference to serenity's threadpool,
/// so that you may run your commands in separate threads.
#[async_trait]
pub trait Framework {
    async fn dispatch(&mut self, _: Context, _: Message);
}

#[async_trait]
impl<F: Framework + ?Sized + Send> Framework for Box<F> {
     #[inline]
    async fn dispatch(&mut self, ctx: Context, msg: Message) {
        (**self).dispatch(ctx, msg).await;
    }
}

#[async_trait]
impl<T: Framework + ?Sized + Send + Sync> Framework for Arc<T> {
    #[inline]
    async fn dispatch(&mut self, ctx: Context, msg: Message) {
        if let Some(s) = Arc::get_mut(self) {
            (*s).dispatch(ctx, msg).await;
        }
    }
}

#[async_trait]
impl<'a, F: Framework + ?Sized + Send> Framework for &'a mut F {
     #[inline]
    async fn dispatch(&mut self, ctx: Context, msg: Message) {
        (**self).dispatch(ctx, msg).await;
    }
}
