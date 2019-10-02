use crate::client::bridge::gateway::ShardMessenger;
use crate::gateway::InterMessage;
use crate::model::prelude::*;
use crate::internal::AsyncRwLock;
use std::sync::Arc;
use typemap::ShareMap;

use crate::http::Http;

#[cfg(feature = "cache")]
pub use crate::cache::{Cache, CacheRwLock};

use futures::channel::mpsc::UnboundedSender;

/// The context is a general utility struct provided on event dispatches, which
/// helps with dealing with the current "context" of the event dispatch.
/// The context also acts as a general high-level interface over the associated
/// [`Shard`] which received the event, or the low-level [`http`] module.
///
/// The context contains "shortcuts", like for interacting with the shard.
/// Methods like [`set_activity`] will unlock the shard and perform an update for
/// you to save a bit of work.
///
/// A context will only live for the event it was dispatched for. After the
/// event handler finished, it is destroyed and will not be re-used.
///
/// [`Shard`]: ../gateway/struct.Shard.html
/// [`http`]: ../http/index.html
/// [`set_activity`]: #method.set_activity
#[derive(Clone)]
pub struct Context {
    /// A clone of [`Client::data`]. Refer to its documentation for more
    /// information.
    ///
    /// [`Client::data`]: struct.Client.html#structfield.data
    pub data: Arc<AsyncRwLock<ShareMap>>,
    /// The messenger to communicate with the shard runner.
    pub shard: ShardMessenger,
    /// The ID of the shard this context is related to.
    pub shard_id: u64,
    pub http: Arc<Http>,
    #[cfg(feature = "cache")]
    pub cache: CacheRwLock,
}

impl Context {
    /// Create a new Context to be passed to an event handler.
    #[cfg(feature = "cache")]
    pub(crate) fn new(
        data: Arc<AsyncRwLock<ShareMap>>,
        runner_tx: UnboundedSender<InterMessage>,
        shard_id: u64,
        http: Arc<Http>,
        cache: Arc<AsyncRwLock<Cache>>,
    ) -> Context {
        Context {
            shard: ShardMessenger::new(runner_tx),
            shard_id,
            data,
            http,
            cache: cache.into(),
        }
    }

    /// Create a new Context to be passed to an event handler.
    #[cfg(not(feature = "cache"))]
    pub(crate) fn new(
        data: Arc<AsyncRwLock<ShareMap>>,
        runner_tx: UnboundedSender<InterMessage>,
        shard_id: u64,
        http: Arc<Http>,
    ) -> Context {
        Context {
            shard: ShardMessenger::new(runner_tx),
            shard_id,
            data,
            http,
        }
    }

    /// Sets the current user as being [`Online`]. This maintains the current
    /// activity.
    ///
    /// # Examples
    ///
    /// Set the current user to being online on the shard:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// #
    /// use async_trait::async_trait;
    ///
    /// struct Handler;
    ///
    /// #[async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, mut ctx: Context, msg: Message) {
    ///         if msg.content == "!online" {
    ///             ctx.online().await;
    ///         }
    ///     }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let mut client = Client::new("token", Handler).await.unwrap();
    ///
    /// client.start().await.unwrap();
    /// # }
    /// ```
    ///
    /// [`Online`]: ../model/user/enum.OnlineStatus.html#variant.Online
    #[inline]
    pub async fn online(&mut self) {
        self.shard.set_status(OnlineStatus::Online).await;
    }

    /// Sets the current user as being [`Idle`]. This maintains the current
    /// activity.
    ///
    /// # Examples
    ///
    /// Set the current user to being idle on the shard:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// #
    /// use async_trait::async_trait;
    ///
    /// struct Handler;
    ///
    /// #[async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, mut ctx: Context, msg: Message) {
    ///         if msg.content == "!idle" {
    ///             ctx.idle().await;
    ///         }
    ///     }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let mut client = Client::new("token", Handler).await.unwrap();
    ///
    /// client.start().await.unwrap();
    /// # }
    /// ```
    ///
    /// [`Idle`]: ../model/user/enum.OnlineStatus.html#variant.Idle
    #[inline]
    pub async fn idle(&mut self) {
        self.shard.set_status(OnlineStatus::Idle).await;
    }

    /// Sets the current user as being [`DoNotDisturb`]. This maintains the
    /// current activity.
    ///
    /// # Examples
    ///
    /// Set the current user to being Do Not Disturb on the shard:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// #
    /// use async_trait::async_trait;
    ///
    /// struct Handler;
    ///
    /// #[async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, mut ctx: Context, msg: Message) {
    ///         if msg.content == "!dnd" {
    ///             ctx.dnd().await;
    ///         }
    ///     }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let mut client = Client::new("token", Handler).await.unwrap();
    ///
    /// client.start().await.unwrap();
    /// # }
    /// ```
    ///
    /// [`DoNotDisturb`]: ../model/user/enum.OnlineStatus.html#variant.DoNotDisturb
    #[inline]
    pub async fn dnd(&mut self) {
        self.shard.set_status(OnlineStatus::DoNotDisturb).await;
    }

    /// Sets the current user as being [`Invisible`]. This maintains the current
    /// activity.
    ///
    /// # Examples
    ///
    /// Set the current user to being invisible on the shard when an
    /// [`Event::Ready`] is received:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::gateway::Ready;
    /// #
    /// use async_trait::async_trait;
    ///
    /// struct Handler;
    ///
    /// #[async_trait]
    /// impl EventHandler for Handler {
    ///     async fn ready(&self, mut ctx: Context, _: Ready) {
    ///         ctx.invisible().await;
    ///     }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let mut client = Client::new("token", Handler).await.unwrap();
    ///
    /// client.start().await.unwrap();
    /// # }
    /// ```
    ///
    /// [`Event::Ready`]: ../model/event/enum.Event.html#variant.Ready
    /// [`Invisible`]: ../model/user/enum.OnlineStatus.html#variant.Invisible
    #[inline]
    pub async fn invisible(&mut self) {
        self.shard.set_status(OnlineStatus::Invisible).await;
    }

    /// "Resets" the current user's presence, by setting the activity to `None`
    /// and the online status to [`Online`].
    ///
    /// Use [`set_presence`] for fine-grained control over individual details.
    ///
    /// # Examples
    ///
    /// Reset the presence when an [`Event::Resumed`] is received:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::event::ResumedEvent;
    /// #
    /// use async_trait::async_trait;
    ///
    /// struct Handler;
    ///
    /// #[async_trait]
    /// impl EventHandler for Handler {
    ///     async fn resume(&self, mut ctx: Context, _: ResumedEvent) {
    ///         ctx.reset_presence().await;
    ///     }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let mut client = Client::new("token", Handler).await.unwrap();
    ///
    /// client.start().await.unwrap();
    /// # }
    /// ```
    ///
    /// [`Event::Resumed`]: ../model/event/enum.Event.html#variant.Resumed
    /// [`Online`]: ../model/user/enum.OnlineStatus.html#variant.Online
    /// [`set_presence`]: #method.set_presence
    #[inline]
    pub async fn reset_presence(&mut self) {
        self.shard.set_presence(None::<Activity>, OnlineStatus::Online).await;
    }

    /// Sets the current activity, defaulting to an online status of [`Online`].
    ///
    /// # Examples
    ///
    /// Create a command named `~setgame` that accepts a name of a game to be
    /// playing:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "model")]
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// #
    /// use serenity::model::gateway::Activity;
    /// use async_trait::async_trait;
    ///
    /// struct Handler;
    ///
    /// #[async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, mut ctx: Context, msg: Message) {
    ///         let args = msg.content.splitn(2, ' ').collect::<Vec<&str>>();
    ///
    ///         if args.len() < 2 || *unsafe { args.get_unchecked(0) } != "~setgame" {
    ///             return;
    ///         }
    ///
    ///         ctx.set_activity(Activity::playing(*unsafe { args.get_unchecked(1) })).await;
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).await.unwrap();
    ///
    /// client.start().await.unwrap();
    /// # }
    ///
    /// # #[cfg(not(feature = "model"))]
    /// # fn main() {}
    /// ```
    ///
    /// [`Online`]: ../model/user/enum.OnlineStatus.html#variant.Online
    #[inline]
    pub async fn set_activity(&mut self, activity: Activity) {
        self.shard.set_presence(Some(activity), OnlineStatus::Online).await;
    }

    /// Sets the current user's presence, providing all fields to be passed.
    ///
    /// # Examples
    ///
    /// Setting the current user as having no activity and being [`Idle`]:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::gateway::Ready;
    /// #
    /// use async_trait::async_trait;
    ///
    /// struct Handler;
    ///
    /// #[async_trait]
    /// impl EventHandler for Handler {
    ///     async fn ready(&self, mut ctx: Context, _: Ready) {
    ///         use serenity::model::user::OnlineStatus;
    ///
    ///         ctx.set_presence(None, OnlineStatus::Idle).await;
    ///     }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let mut client = Client::new("token", Handler).await.unwrap();
    ///
    /// client.start().await.unwrap();
    /// # }
    /// ```
    ///
    /// Setting the current user as playing `"Heroes of the Storm"`, while being
    /// [`DoNotDisturb`]:
    ///
    /// ```rust,ignore
    /// # use serenity::prelude::*;
    /// # use serenity::model::gateway::Ready;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn ready(&self, context: Context, _: Ready) {
    ///         use serenity::model::gateway::Activity;
    ///         use serenity::model::user::OnlineStatus;
    ///
    ///         let activity = Activity::playing("Heroes of the Storm");
    ///         let status = OnlineStatus::DoNotDisturb;
    ///
    ///         context.set_presence(Some(activity), status);
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    ///
    /// [`DoNotDisturb`]: ../model/user/enum.OnlineStatus.html#variant.DoNotDisturb
    /// [`Idle`]: ../model/user/enum.OnlineStatus.html#variant.Idle
    #[inline]
    pub async fn set_presence(&mut self, activity: Option<Activity>, status: OnlineStatus) {
        self.shard.set_presence(activity, status).await;
    }
}

impl AsRef<Http> for Context {
    fn as_ref(&self) -> &Http { &self.http }
}

#[cfg(feature = "cache")]
impl AsRef<CacheRwLock> for Context {
    fn as_ref(&self) -> &CacheRwLock {
        &self.cache
    }
}
