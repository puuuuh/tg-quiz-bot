#[derive(Clone,Hash,PartialEq,Eq)]
pub enum MessageKind {
    /// Text message.
    Text,
    /// Message is an audio file.
    Audio,
    /// Message is a general file.
    Document,
    /// Message is a photo.
    Photo,
    /// Message is a sticker.
    Sticker,
    /// Message is a video.
    Video,
    /// Message is a voice message.
    Voice,
    /// Message is a video note.
    VideoNote,
    /// Message is a shared contact.
    Contact,
    /// Message is a shared location.
    Location,
    /// Message is a poll.
    Poll,
    /// Message is a venue.
    Venue,
    /// New members that were added to the group or supergroup and
    /// information about them (the bot itself may be one of these members)
    NewChatMembers,
    /// A member was removed from the group.
    LeftChatMember,
    /// New chat title.
    NewChatTitle,
    /// New chat photo.
    NewChatPhoto,
    /// Service message: the chat photo was deleted.
    DeleteChatPhoto,
    /// Service message: the group has been created.
    GroupChatCreated,
    /// Service message: the supergroup has been created. This field can‘t be received in a
    /// message coming through updates, because bot can’t be a member of a supergroup when
    /// it is created. It can only be found in reply_to_message if someone replies to a very
    /// first message in a directly created supergroup.
    SupergroupChatCreated,
    /// Service message: the channel has been created. This field can‘t be received in a message
    /// coming through updates, because bot can’t be a member of a channel when it is created.
    /// It can only be found in reply_to_message if someone replies
    /// to a very first message in a channel.
    ChannelChatCreated,
    /// The group has been migrated to a supergroup.
    MigrateToChatId,
    /// The supergroup has been migrated from a group.
    MigrateFromChatId,
    /// Specified message was pinned.
    PinnedMessage,
    #[doc(hidden)]
    Unknown,
}

impl From<&telegram_bot::MessageKind> for MessageKind {
    fn from(k: &telegram_bot::MessageKind) -> Self {
        match k {
            telegram_bot::MessageKind::Text { .. } => {MessageKind::Text},
            telegram_bot::MessageKind::Audio { .. } => {MessageKind::Audio},
            telegram_bot::MessageKind::Document { .. } => {MessageKind::Document},
            telegram_bot::MessageKind::Photo { .. } => {MessageKind::Photo},
            telegram_bot::MessageKind::Sticker { .. } => {MessageKind::Sticker},
            telegram_bot::MessageKind::Video { .. } => {MessageKind::Video},
            telegram_bot::MessageKind::Voice { .. } => {MessageKind::Voice},
            telegram_bot::MessageKind::VideoNote { .. } => {MessageKind::VideoNote},
            telegram_bot::MessageKind::Contact { .. } => {MessageKind::Contact},
            telegram_bot::MessageKind::Location { .. } => {MessageKind::Location},
            telegram_bot::MessageKind::Poll { .. } => {MessageKind::Poll},
            telegram_bot::MessageKind::Venue { .. } => {MessageKind::Venue},
            telegram_bot::MessageKind::NewChatMembers { .. } => {MessageKind::NewChatMembers},
            telegram_bot::MessageKind::LeftChatMember { .. } => {MessageKind::LeftChatMember},
            telegram_bot::MessageKind::NewChatTitle { .. } => {MessageKind::NewChatTitle},
            telegram_bot::MessageKind::NewChatPhoto { .. } => {MessageKind::NewChatPhoto},
            telegram_bot::MessageKind::DeleteChatPhoto => {MessageKind::DeleteChatPhoto},
            telegram_bot::MessageKind::GroupChatCreated => {MessageKind::GroupChatCreated},
            telegram_bot::MessageKind::SupergroupChatCreated => {MessageKind::SupergroupChatCreated},
            telegram_bot::MessageKind::ChannelChatCreated => {MessageKind::ChannelChatCreated},
            telegram_bot::MessageKind::MigrateToChatId { .. } => {MessageKind::MigrateToChatId},
            telegram_bot::MessageKind::MigrateFromChatId { .. } => {MessageKind::MigrateFromChatId},
            telegram_bot::MessageKind::PinnedMessage { .. } => {MessageKind::PinnedMessage},
            telegram_bot::MessageKind::Unknown { .. } => {MessageKind::Unknown},
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum UpdateKind {
    Message,
    EditedMessage,
    ChannelPost,
    EditedChannelPost,
    InlineQuery,
    CallbackQuery,
    Poll,
    PollAnswer,
    Error,
    Unknown,
}

impl From<&telegram_bot::UpdateKind> for UpdateKind {
    fn from(k: &telegram_bot::UpdateKind) -> Self {
        match k {
            telegram_bot::UpdateKind::Message(_) => {UpdateKind::Message},
            telegram_bot::UpdateKind::EditedMessage(_) => {UpdateKind::EditedMessage},
            telegram_bot::UpdateKind::ChannelPost(_) => {UpdateKind::ChannelPost},
            telegram_bot::UpdateKind::EditedChannelPost(_) => {UpdateKind::EditedChannelPost},
            telegram_bot::UpdateKind::InlineQuery(_) => {UpdateKind::InlineQuery},
            telegram_bot::UpdateKind::CallbackQuery(_) => {UpdateKind::CallbackQuery},
            telegram_bot::UpdateKind::Poll(_) => {UpdateKind::Poll},
            telegram_bot::UpdateKind::PollAnswer(_) => {UpdateKind::PollAnswer},
            telegram_bot::UpdateKind::Error(_) => {UpdateKind::Error},
            telegram_bot::UpdateKind::Unknown => {UpdateKind::Unknown},
        }
    }
}
