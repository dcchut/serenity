use crate::gateway::InterMessage;
use crate::model::prelude::*;
use super::{ShardClientMessage, ShardRunnerMessage};
use tungstenite::Message;
use futures::channel::mpsc::{UnboundedSender, SendError};
use futures::sink::SinkExt;

/// A lightweight wrapper around an mpsc sender.
///
/// This is used to cleanly communicate with a shard's respective
/// [`ShardRunner`]. This can be used for actions such as setting the activity
/// via [`set_activity`] or shutting down via [`shutdown`].
///
/// [`ShardRunner`]: struct.ShardRunner.html
/// [`set_activity`]: #method.set_activity
/// [`shutdown`]: #method.shutdown
#[derive(Clone, Debug)]
pub struct ShardMessenger {
    tx: UnboundedSender<InterMessage>,
}

impl ShardMessenger {
    /// Creates a new shard messenger.
    ///
    /// If you are using the [`Client`], you do not need to do this.
    ///
    /// [`Client`]: ../../struct.Client.html
    #[inline]
    pub fn new(tx: UnboundedSender<InterMessage>) -> Self {
        Self {
            tx,
        }
    }

    /// Requests that one or multiple [`Guild`]s be chunked.
    ///
    /// This will ask the gateway to start sending member chunks for large
    /// guilds (250 members+). If a guild is over 250 members, then a full
    /// member list will not be downloaded, and must instead be requested to be
    /// sent in "chunks" containing members.
    ///
    /// Member chunks are sent as the [`Event::GuildMembersChunk`] event. Each
    /// chunk only contains a partial amount of the total members.
    ///
    /// If the `cache` feature is enabled, the cache will automatically be
    /// updated with member chunks.
    ///
    /// # Examples
    ///
    /// Chunk a single guild by Id, limiting to 2000 [`Member`]s, and not
    /// specifying a query parameter:
    ///
    /// ```rust,no_run
    /// # use parking_lot::Mutex;
    /// # use serenity::gateway::Shard;
    /// # use std::error::Error;
    /// # use std::sync::Arc;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let mutex = Arc::new(Mutex::new("".to_string()));
    /// #
    /// #     let mut shard = Shard::new(mutex.clone(), "", [0, 1], true)?;
    /// #
    /// use serenity::model::id::GuildId;
    ///
    /// let guild_ids = vec![GuildId(81384788765712384)];
    ///
    /// shard.chunk_guilds(guild_ids, Some(2000), None);
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// Chunk a single guild by Id, limiting to 20 members, and specifying a
    /// query parameter of `"do"`:
    ///
    /// ```rust,no_run
    /// # use parking_lot::Mutex;
    /// # use serenity::gateway::Shard;
    /// # use std::error::Error;
    /// # use std::sync::Arc;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let mutex = Arc::new(Mutex::new("".to_string()));
    /// #
    /// #     let mut shard = Shard::new(mutex.clone(), "", [0, 1], true)?;
    /// #
    /// use serenity::model::id::GuildId;
    ///
    /// let guild_ids = vec![GuildId(81384788765712384)];
    ///
    /// shard.chunk_guilds(guild_ids, Some(20), Some("do"));
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// [`Event::GuildMembersChunk`]: ../../../model/event/enum.Event.html#variant.GuildMembersChunk
    /// [`Guild`]: ../../../model/guild/struct.Guild.html
    /// [`Member`]: ../../../model/guild/struct.Member.html
    pub async fn chunk_guilds<It>(
        &mut self,
        guild_ids: It,
        limit: Option<u16>,
        query: Option<String>,
    ) where It: IntoIterator<Item=GuildId> {
        let guilds = guild_ids.into_iter().collect::<Vec<GuildId>>();

        let _ = self.send(ShardRunnerMessage::ChunkGuilds {
            guild_ids: guilds,
            limit,
            query,
        }).await;
    }

    /// Sets the user's current activity, if any.
    ///
    /// Other presence settings are maintained.
    ///
    /// # Examples
    ///
    /// Setting the current activity to playing `"Heroes of the Storm"`:
    ///
    /// ```rust,no_run
    /// # use parking_lot::Mutex;
    /// # use serenity::gateway::Shard;
    /// # use std::error::Error;
    /// # use std::sync::Arc;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let mutex = Arc::new(Mutex::new("".to_string()));
    /// #
    /// #     let mut shard = Shard::new(mutex.clone(), "", [0, 1], true)?;
    /// use serenity::model::gateway::Activity;
    ///
    /// shard.set_activity(Some(Activity::playing("Heroes of the Storm")));
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    pub async fn set_activity(&mut self, activity: Option<Activity>) {
        let _ = self.send(ShardRunnerMessage::SetActivity(activity)).await;
    }

    /// Sets the user's full presence information.
    ///
    /// Consider using the individual setters if you only need to modify one of
    /// these.
    ///
    /// # Examples
    ///
    /// Set the current user as playing `"Heroes of the Storm"` and being
    /// online:
    ///
    /// ```rust,ignore
    /// # use parking_lot::Mutex;
    /// # use serenity::gateway::Shard;
    /// # use std::error::Error;
    /// # use std::sync::Arc;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let mutex = Arc::new(Mutex::new("".to_string()));
    /// #
    /// #     let mut shard = Shard::new(mutex.clone(), "", [0, 1], true)?;
    /// #
    /// use serenity::model::{Activity, OnlineStatus};
    ///
    /// shard.set_presence(Some(Activity::playing("Heroes of the Storm")), OnlineStatus::Online);
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    pub async fn set_presence(&mut self, activity: Option<Activity>, mut status: OnlineStatus) {
        if status == OnlineStatus::Offline {
            status = OnlineStatus::Invisible;
        }

        let _ = self.send(ShardRunnerMessage::SetPresence(status, activity)).await;
    }

    /// Sets the user's current online status.
    ///
    /// Note that [`Offline`] is not a valid online status, so it is
    /// automatically converted to [`Invisible`].
    ///
    /// Other presence settings are maintained.
    ///
    /// # Examples
    ///
    /// Setting the current online status for the shard to [`DoNotDisturb`].
    ///
    /// ```rust,no_run
    /// # use parking_lot::Mutex;
    /// # use serenity::gateway::Shard;
    /// # use std::error::Error;
    /// # use std::sync::Arc;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let mutex = Arc::new(Mutex::new("".to_string()));
    /// #
    /// #     let mut shard = Shard::new(mutex.clone(), "", [0, 1], true)?;
    /// #
    /// use serenity::model::user::OnlineStatus;
    ///
    /// shard.set_status(OnlineStatus::DoNotDisturb);
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// [`DoNotDisturb`]: ../../../model/user/enum.OnlineStatus.html#variant.DoNotDisturb
    /// [`Invisible`]: ../../../model/user/enum.OnlineStatus.html#variant.Invisible
    /// [`Offline`]: ../../../model/user/enum.OnlineStatus.html#variant.Offline
    pub async fn set_status(&mut self, mut online_status: OnlineStatus) {
        if online_status == OnlineStatus::Offline {
            online_status = OnlineStatus::Invisible;
        }

        let _ = self.send(ShardRunnerMessage::SetStatus(online_status)).await;
    }

    /// Shuts down the websocket by attempting to cleanly close the
    /// connection.
    pub async fn shutdown_clean(&mut self) {
        let _ = self.send(ShardRunnerMessage::Close(1000, None)).await;
    }

    /// Sends a raw message over the WebSocket.
    ///
    /// The given message is not mutated in any way, and is sent as-is.
    ///
    /// You should only use this if you know what you're doing. If you're
    /// wanting to, for example, send a presence update, prefer the usage of
    /// the [`set_presence`] method.
    ///
    /// [`set_presence`]: #method.set_presence
    pub async fn websocket_message(&mut self, message: Message) {
        let _ = self.send(ShardRunnerMessage::Message(message)).await;
    }

    #[inline]
    async fn send(&mut self, msg: ShardRunnerMessage)
        -> Result<(), SendError> {
        self.tx.send(InterMessage::Client(Box::new(ShardClientMessage::Runner(msg)))).await
    }
}
