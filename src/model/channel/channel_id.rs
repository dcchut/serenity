#[cfg(feature = "http")]
use crate::http::CacheHttp;
use crate::model::prelude::*;

#[cfg(feature = "model")]
use crate::builder::{CreateMessage, EditChannel, EditMessage, GetMessages};
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::{Cache, CacheRwLock};
#[cfg(feature = "model")]
use crate::http::AttachmentType;
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::internal::AsyncRwLock;
#[cfg(feature = "model")]
use crate::utils;
#[cfg(all(feature = "http", feature = "model"))]
use serde_json::json;
#[cfg(feature = "model")]
use std::borrow::Cow;
#[cfg(feature = "model")]
use std::fmt::Write as FmtWrite;

#[cfg(feature = "model")]
impl ChannelId {
    /// Broadcasts that the current user is typing to a channel for the next 5
    /// seconds.
    ///
    /// After 5 seconds, another request must be made to continue broadcasting
    /// that the current user is typing.
    ///
    /// This should rarely be used for bots, and should likely only be used for
    /// signifying that a long-running command is still being executed.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use serenity::model::ChannelId;
    ///
    /// let _successful = ChannelId(7).broadcast_typing();
    /// ```
    ///
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    #[cfg(feature = "http")]
    #[inline]
    pub async fn broadcast_typing(self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().broadcast_typing(self.0).await
    }

    /// Creates a [permission overwrite][`PermissionOverwrite`] for either a
    /// single [`Member`] or [`Role`] within the channel.
    ///
    /// Refer to the documentation for [`GuildChannel::create_permission`] for
    /// more information.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// [`GuildChannel::create_permission`]: ../channel/struct.GuildChannel.html#method.create_permission
    /// [`Member`]: ../guild/struct.Member.html
    /// [`PermissionOverwrite`]: ../channel/struct.PermissionOverwrite.html
    /// [`Role`]: ../guild/struct.Role.html
    /// [Manage Channels]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn create_permission(
        self,
        http: impl AsRef<Http>,
        target: &PermissionOverwrite,
    ) -> Result<()> {
        let (id, kind) = match target.kind {
            PermissionOverwriteType::Member(id) => (id.0, "member"),
            PermissionOverwriteType::Role(id) => (id.0, "role"),
        };

        let map = json!({
            "allow": target.allow.bits(),
            "deny": target.deny.bits(),
            "id": id,
            "type": kind,
        });

        http.as_ref().create_permission(self.0, id, &map).await
    }

    /// React to a [`Message`] with a custom [`Emoji`] or unicode character.
    ///
    /// [`Message::react`] may be a more suited method of reacting in most
    /// cases.
    ///
    /// Requires the [Add Reactions] permission, _if_ the current user is the
    /// first user to perform a react with a certain emoji.
    ///
    /// [`Emoji`]: ../guild/struct.Emoji.html
    /// [`Message`]: ../channel/struct.Message.html
    /// [`Message::react`]: ../channel/struct.Message.html#method.react
    /// [Add Reactions]: ../permissions/struct.Permissions.html#associatedconstant.ADD_REACTIONS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn create_reaction<M, R>(
        self,
        http: impl AsRef<Http>,
        message_id: M,
        reaction_type: R,
    ) -> Result<()>
    where
        M: Into<MessageId>,
        R: Into<ReactionType>,
    {
        let reaction_type = reaction_type.into();

        self._create_reaction(&http, message_id.into(), &reaction_type)
            .await
    }

    async fn _create_reaction(
        self,
        http: impl AsRef<Http>,
        message_id: MessageId,
        reaction_type: &ReactionType,
    ) -> Result<()> {
        http.as_ref()
            .create_reaction(self.0, message_id.0, reaction_type)
            .await
    }

    /// Deletes this channel, returning the channel on a successful deletion.
    #[cfg(feature = "http")]
    #[inline]
    pub async fn delete(self, http: impl AsRef<Http>) -> Result<Channel> {
        http.as_ref().delete_channel(self.0).await
    }

    /// Deletes a [`Message`] given its Id.
    ///
    /// Refer to [`Message::delete`] for more information.
    ///
    /// Requires the [Manage Messages] permission, if the current user is not
    /// the author of the message.
    ///
    /// [`Message`]: ../channel/struct.Message.html
    /// [`Message::delete`]: ../channel/struct.Message.html#method.delete
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[cfg(feature = "http")]
    #[inline]
    pub async fn delete_message<M: Into<MessageId>>(
        self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<()> {
        self._delete_message(&http, message_id.into()).await
    }

    #[cfg(feature = "http")]
    async fn _delete_message(self, http: impl AsRef<Http>, message_id: MessageId) -> Result<()> {
        http.as_ref().delete_message(self.0, message_id.0).await
    }

    /// Deletes all messages by Ids from the given vector in the given channel.
    ///
    /// Refer to the documentation for [`Channel::delete_messages`] for more
    /// information.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// **Note**: Messages that are older than 2 weeks can't be deleted using
    /// this method.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::BulkDeleteAmount`] if an attempt was made to
    /// delete either 0 or more than 100 messages.
    ///
    /// [`Channel::delete_messages`]: ../channel/enum.Channel.html#method.delete_messages
    /// [`ModelError::BulkDeleteAmount`]: ../error/enum.Error.html#variant.BulkDeleteAmount
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[cfg(feature = "http")]
    pub async fn delete_messages<T: AsRef<MessageId>, It: IntoIterator<Item = T>>(
        self,
        http: impl AsRef<Http>,
        message_ids: It,
    ) -> Result<()> {
        let ids = message_ids
            .into_iter()
            .map(|message_id| message_id.as_ref().0)
            .collect::<Vec<u64>>();

        self._delete_messages(&http, &ids).await
    }

    #[cfg(feature = "http")]
    async fn _delete_messages(self, http: impl AsRef<Http>, ids: &[u64]) -> Result<()> {
        let len = ids.len();

        if len == 0 || len > 100 {
            Err(Error::Model(ModelError::BulkDeleteAmount))
        } else if ids.len() == 1 {
            self.delete_message(&http, ids[0]).await
        } else {
            let map = json!({ "messages": ids });

            http.as_ref().delete_messages(self.0, &map).await
        }
    }

    /// Deletes all permission overrides in the channel from a member or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[cfg(feature = "http")]
    pub async fn delete_permission(
        self,
        http: impl AsRef<Http>,
        permission_type: PermissionOverwriteType,
    ) -> Result<()> {
        http.as_ref()
            .delete_permission(
                self.0,
                match permission_type {
                    PermissionOverwriteType::Member(id) => id.0,
                    PermissionOverwriteType::Role(id) => id.0,
                },
            )
            .await
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// [`Reaction`]: ../channel/struct.Reaction.html
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[cfg(feature = "http")]
    #[inline]
    pub async fn delete_reaction<M, R>(
        self,
        http: impl AsRef<Http>,
        message_id: M,
        user_id: Option<UserId>,
        reaction_type: R,
    ) -> Result<()>
    where
        M: Into<MessageId>,
        R: Into<ReactionType>,
    {
        let reaction_type = reaction_type.into();

        self._delete_reaction(&http, message_id.into(), user_id, &reaction_type)
            .await
    }

    #[cfg(feature = "http")]
    async fn _delete_reaction(
        self,
        http: impl AsRef<Http>,
        message_id: MessageId,
        user_id: Option<UserId>,
        reaction_type: &ReactionType,
    ) -> Result<()> {
        http.as_ref()
            .delete_reaction(
                self.0,
                message_id.0,
                user_id.map(|uid| uid.0),
                reaction_type,
            )
            .await
    }

    /// Edits the settings of a [`Channel`], optionally setting new values.
    ///
    /// Refer to `EditChannel`'s documentation for its methods.
    ///
    /// Requires the [Manage Channel] permission.
    ///
    /// # Examples
    ///
    /// Change a voice channel's name and bitrate:
    ///
    /// ```rust,ignore
    /// // assuming a `channel_id` has been bound
    ///
    /// channel_id.edit(&context, |c| c.name("test").bitrate(64000));
    /// ```
    ///
    /// [`Channel`]: ../channel/enum.Channel.html
    /// [Manage Channel]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[cfg(all(feature = "utils", feature = "http"))]
    #[inline]
    pub async fn edit<F: FnOnce(&mut EditChannel) -> &mut EditChannel>(
        self,
        http: impl AsRef<Http>,
        f: F,
    ) -> Result<GuildChannel> {
        let mut channel = EditChannel::default();
        f(&mut channel);

        let map = utils::hashmap_to_json_map(channel.0);

        http.as_ref().edit_channel(self.0, &map).await
    }

    /// Edits a [`Message`] in the channel given its Id.
    ///
    /// Message editing preserves all unchanged message data.
    ///
    /// Refer to the documentation for [`EditMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// **Note**: Requires that the current user be the author of the message.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the [`the limit`], containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ModelError::MessageTooLong`]: ../error/enum.Error.html#variant.MessageTooLong
    /// [`EditMessage`]: ../../builder/struct.EditMessage.html
    /// [`Message`]: ../channel/struct.Message.html
    /// [`the limit`]: ../../builder/struct.EditMessage.html#method.content
    #[cfg(all(feature = "utils", feature = "http"))]
    #[inline]
    pub async fn edit_message<F, M>(
        self,
        http: impl AsRef<Http>,
        message_id: M,
        f: F,
    ) -> Result<Message>
    where
        F: FnOnce(&mut EditMessage) -> &mut EditMessage,
        M: Into<MessageId>,
    {
        self._edit_message(&http, message_id.into(), f).await
    }

    async fn _edit_message<F>(
        self,
        http: impl AsRef<Http>,
        message_id: MessageId,
        f: F,
    ) -> Result<Message>
    where
        F: FnOnce(&mut EditMessage) -> &mut EditMessage,
    {
        let mut msg = EditMessage::default();
        f(&mut msg);

        if let Some(content) = msg.0.get(&"content") {
            if let Value::String(ref content) = *content {
                if let Some(length_over) = Message::overflow_length(content) {
                    return Err(Error::Model(ModelError::MessageTooLong(length_over)));
                }
            }
        }

        let map = utils::hashmap_to_json_map(msg.0);
        let obj = Value::Object(map);

        http.as_ref().edit_message(self.0, message_id.0, &obj).await
    }

    /// Attempts to find a [`Channel`] by its Id in the cache.
    ///
    /// [`Channel`]: ../channel/enum.Channel.html
    #[cfg(feature = "cache")]
    #[inline]
    pub async fn to_channel_cached(self, cache: impl AsRef<CacheRwLock>) -> Option<Channel> {
        self._to_channel_cached(&cache.as_ref()).await
    }

    /// To allow testing pass their own cache instead of using the globale one.
    #[cfg(feature = "cache")]
    #[inline]
    pub(crate) async fn _to_channel_cached(self, cache: &AsyncRwLock<Cache>) -> Option<Channel> {
        cache.read().await.channel(self)
    }

    /// First attempts to find a [`Channel`] by its Id in the cache,
    /// upon failure requests it via the REST API.
    ///
    /// **Note**: If the `cache`-feature is enabled permissions will be checked and upon
    /// owning the required permissions the HTTP-request will be issued.
    ///
    /// [`Channel`]: ../channel/enum.Channel.html
    #[cfg(feature = "http")]
    #[inline]
    pub async fn to_channel(self, cache_http: impl CacheHttp) -> Result<Channel> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(channel) = cache.read().await.channel(self) {
                    return Ok(channel);
                }
            }
        }

        cache_http.http().get_channel(self.0).await
    }

    /// Gets all of the channel's invites.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// [Manage Channels]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn invites(self, http: impl AsRef<Http>) -> Result<Vec<RichInvite>> {
        http.as_ref().get_channel_invites(self.0).await
    }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: ../permissions/struct.Permissions.html#associatedconstant.READ_MESSAGE_HISTORY
    #[cfg(feature = "http")]
    #[inline]
    pub async fn message<M: Into<MessageId>>(
        self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<Message> {
        self._message(&http, message_id.into()).await
    }

    #[cfg(feature = "http")]
    async fn _message(self, http: impl AsRef<Http>, message_id: MessageId) -> Result<Message> {
        let mut msg = http.as_ref().get_message(self.0, message_id.0).await?;
        msg.transform_content().await;

        Ok(msg)
    }

    /// Gets messages from the channel.
    ///
    /// Refer to [`GetMessages`] for more information on how to use `builder`.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [`GetMessages`]: ../../builder/struct.GetMessages.html
    /// [Read Message History]: ../permissions/struct.Permissions.html#associatedconstant.READ_MESSAGE_HISTORY
    #[cfg(feature = "http")]
    pub async fn messages<F>(self, http: impl AsRef<Http>, builder: F) -> Result<Vec<Message>>
    where
        F: FnOnce(&mut GetMessages) -> &mut GetMessages,
    {
        let mut get_messages = GetMessages::default();
        builder(&mut get_messages);
        let mut map = get_messages.0;
        let mut query = format!("?limit={}", map.remove(&"limit").unwrap_or(50));

        if let Some(after) = map.remove(&"after") {
            write!(query, "&after={}", after)?;
        } else if let Some(around) = map.remove(&"around") {
            write!(query, "&around={}", around)?;
        } else if let Some(before) = map.remove(&"before") {
            write!(query, "&before={}", before)?;
        }

        let msg = http.as_ref().get_messages(self.0, &query).await?;
        let mut _msg = Vec::with_capacity(msg.len());

        // TODO: compare this to master to make sure I didn't screw it up royally
        for mut msgs in msg {
            msgs.transform_content().await;
            _msg.push(msgs);
        }

        Ok(_msg)
    }

    /// Returns the name of whatever channel this id holds.
    #[cfg(all(feature = "model", feature = "cache"))]
    pub async fn name(self, cache: impl AsRef<CacheRwLock>) -> Option<String> {
        let channel = self.to_channel_cached(&cache).await?;

        Some(match channel {
            Channel::Guild(channel) => channel.read().await.name().to_string(),
            Channel::Group(channel) => {
                let guard = channel.read().await;
                let res = guard.name();

                match res {
                    Cow::Borrowed(name) => name.to_string(),
                    Cow::Owned(name) => name,
                }
            }
            Channel::Category(category) => category.read().await.name().to_string(),
            Channel::Private(channel) => {
                let guard = channel.read().await;
                guard.name()
            }
        })
    }

    /// Pins a [`Message`] to the channel.
    ///
    /// [`Message`]: ../channel/struct.Message.html
    #[cfg(feature = "http")]
    #[inline]
    pub async fn pin<M: Into<MessageId>>(
        self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<()> {
        self._pin(&http, message_id.into()).await
    }

    async fn _pin(self, http: impl AsRef<Http>, message_id: MessageId) -> Result<()> {
        http.as_ref().pin_message(self.0, message_id.0).await
    }

    /// Gets the list of [`Message`]s which are pinned to the channel.
    ///
    /// [`Message`]: ../channel/struct.Message.html
    #[cfg(feature = "http")]
    #[inline]
    pub async fn pins(self, http: impl AsRef<Http>) -> Result<Vec<Message>> {
        http.as_ref().get_pins(self.0).await
    }

    /// Gets the list of [`User`]s who have reacted to a [`Message`] with a
    /// certain [`Emoji`].
    ///
    /// Refer to [`Channel::reaction_users`] for more information.
    ///
    /// **Note**: Requires the [Read Message History] permission.
    ///
    /// [`Channel::reaction_users`]: ../channel/enum.Channel.html#method.reaction_users
    /// [`Emoji`]: ../guild/struct.Emoji.html
    /// [`Message`]: ../channel/struct.Message.html
    /// [`User`]: ../user/struct.User.html
    /// [Read Message History]: ../permissions/struct.Permissions.html#associatedconstant.READ_MESSAGE_HISTORY
    #[cfg(feature = "http")]
    pub async fn reaction_users<M, R, U>(
        self,
        http: impl AsRef<Http>,
        message_id: M,
        reaction_type: R,
        limit: Option<u8>,
        after: U,
    ) -> Result<Vec<User>>
    where
        M: Into<MessageId>,
        R: Into<ReactionType>,
        U: Into<Option<UserId>>,
    {
        let reaction_type = reaction_type.into();

        self._reaction_users(http, message_id.into(), &reaction_type, limit, after.into())
            .await
    }

    #[cfg(feature = "http")]
    async fn _reaction_users(
        self,
        http: impl AsRef<Http>,
        message_id: MessageId,
        reaction_type: &ReactionType,
        limit: Option<u8>,
        after: Option<UserId>,
    ) -> Result<Vec<User>> {
        let limit = limit.map_or(50, |x| if x > 100 { 100 } else { x });

        http.as_ref()
            .get_reaction_users(
                self.0,
                message_id.0,
                reaction_type,
                limit,
                after.map(|x| x.0),
            )
            .await
    }

    /// Sends a message with just the given message content in the channel.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ChannelId`]: struct.ChannelId.html
    /// [`ModelError::MessageTooLong`]: ../error/enum.Error.html#variant.MessageTooLong
    #[cfg(feature = "http")]
    #[inline]
    pub async fn say(
        self,
        http: impl AsRef<Http>,
        content: impl std::fmt::Display,
    ) -> Result<Message> {
        self.send_message(&http, |m| m.content(content)).await
    }

    /// Sends a file along with optional message contents. The filename _must_
    /// be specified.
    ///
    /// Message contents may be passed by using the [`CreateMessage::content`]
    /// method.
    ///
    /// The [Attach Files] and [Send Messages] permissions are required.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Examples
    ///
    /// Send files with the paths `/path/to/file.jpg` and `/path/to/file2.jpg`:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # let http = Arc::new(Http::default());
    /// use serenity::model::id::ChannelId;
    ///
    /// let channel_id = ChannelId(7);
    ///
    /// let paths = vec!["/path/to/file.jpg", "path/to/file2.jpg"];
    ///
    /// let _ = channel_id.send_files(&http, paths, |m| {
    ///     m.content("a file")
    /// });
    /// ```
    ///
    /// Send files using `File`:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # let http = Arc::new(Http::default());
    /// use serenity::model::id::ChannelId;
    /// use std::fs::File;
    ///
    /// let channel_id = ChannelId(7);
    ///
    /// let f1 = File::open("my_file.jpg").unwrap();
    /// let f2 = File::open("my_file2.jpg").unwrap();
    ///
    /// let files = vec![(&f1, "my_file.jpg"), (&f2, "my_file2.jpg")];
    ///
    /// let _ = channel_id.send_files(&http, files, |m| {
    ///     m.content("a file")
    /// });
    /// ```
    ///
    /// # Errors
    ///
    /// If the content of the message is over the above limit, then a
    /// [`ClientError::MessageTooLong`] will be returned, containing the number
    /// of unicode code points over the limit.
    ///
    /// Returns an
    /// [`HttpError::InvalidRequest(PayloadTooLarge)`][`HttpError::InvalidRequest`]
    /// if the file is too large to send.
    ///
    /// [`ClientError::MessageTooLong`]: ../../client/enum.ClientError.html#variant.MessageTooLong
    /// [`HttpError::InvalidRequest`]: ../../http/enum.HttpError.html#variant.InvalidRequest
    /// [`CreateMessage::content`]: ../../builder/struct.CreateMessage.html#method.content
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [Attach Files]: ../permissions/struct.Permissions.html#associatedconstant.ATTACH_FILES
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    #[cfg(all(feature = "utils", feature = "http"))]
    pub async fn send_files<'a, F, T, It>(
        self,
        http: impl AsRef<Http>,
        files: It,
        f: F,
    ) -> Result<Message>
    where
        for<'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
        T: Into<AttachmentType<'a>>,
        It: IntoIterator<Item = T>,
    {
        let mut create_message = CreateMessage::default();
        let msg = f(&mut create_message);

        if let Some(content) = msg.0.get(&"content") {
            if let Value::String(ref content) = *content {
                if let Some(length_over) = Message::overflow_length(content) {
                    return Err(Error::Model(ModelError::MessageTooLong(length_over)));
                }
            }
        }

        if let Some(e) = msg.0.remove(&"embed") {
            msg.0.insert("payload_json", json!({ "embed": e }));
        }

        let map = utils::hashmap_to_json_map(msg.0.clone());
        http.as_ref().send_files(self.0, files, map).await
    }

    /// Sends a message to the channel.
    ///
    /// Refer to the documentation for [`CreateMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// Requires the [Send Messages] permission.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`Channel`]: ../channel/enum.Channel.html
    /// [`ModelError::MessageTooLong`]: ../error/enum.Error.html#variant.MessageTooLong
    /// [`CreateMessage`]: ../../builder/struct.CreateMessage.html
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    #[cfg(all(feature = "utils", feature = "http"))]
    pub async fn send_message<'a, F>(self, http: impl AsRef<Http>, f: F) -> Result<Message>
    where
        for<'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
    {
        let mut create_message = CreateMessage::default();
        let msg = f(&mut create_message);

        if !msg.2.is_empty() {
            if let Some(e) = msg.0.remove(&"embed") {
                if let Some(c) = msg.0.remove(&"content") {
                    msg.0
                        .insert("payload_json", json!({ "content": c, "embed": e }));
                } else {
                    msg.0.insert("payload_json", json!({ "embed": e }));
                }
            }
        }

        let map = utils::hashmap_to_json_map(msg.0.clone());

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        let message = if msg.2.is_empty() {
            let obj = Value::Object(map);
            http.as_ref().send_message(self.0, &obj).await?
        } else {
            http.as_ref().send_files(self.0, msg.2.clone(), map).await?
        };

        if let Some(reactions) = msg.1.clone() {
            for reaction in reactions {
                self.create_reaction(&http, message.id, reaction).await?;
            }
        }

        Ok(message)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Message`]: ../channel/struct.Message.html
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[cfg(feature = "http")]
    #[inline]
    pub async fn unpin<M: Into<MessageId>>(
        self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<()> {
        self._unpin(&http, message_id.into()).await
    }

    #[cfg(feature = "http")]
    async fn _unpin(self, http: impl AsRef<Http>, message_id: MessageId) -> Result<()> {
        http.as_ref().unpin_message(self.0, message_id.0).await
    }

    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_WEBHOOKS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn webhooks(self, http: impl AsRef<Http>) -> Result<Vec<Webhook>> {
        http.as_ref().get_channel_webhooks(self.0).await
    }
}

impl From<PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: PrivateChannel) -> ChannelId {
        private_channel.id
    }
}

impl<'a> From<&'a PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: &PrivateChannel) -> ChannelId {
        private_channel.id
    }
}

impl From<GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: GuildChannel) -> ChannelId {
        public_channel.id
    }
}

impl<'a> From<&'a GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: &GuildChannel) -> ChannelId {
        public_channel.id
    }
}
