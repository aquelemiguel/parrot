use serenity::model::{
    guild::Guild,
    id::{ChannelId, UserId},
};

pub enum Connection {
    User(ChannelId),
    Bot(ChannelId),
    Mutual(ChannelId, ChannelId),
    Separate(ChannelId, ChannelId),
    Neither,
}

pub fn check_voice_connections(guild: &Guild, user_id: &UserId, bot_id: &UserId) -> Connection {
    let user_channel = get_voice_channel_for_user(guild, user_id);
    let bot_channel = get_voice_channel_for_user(guild, bot_id);

    match (bot_channel, user_channel) {
        (Some(bot_id), Some(user_id)) => {
            if bot_id.0 == user_id.0 {
                Connection::Mutual(bot_id, user_id)
            } else {
                Connection::Separate(bot_id, user_id)
            }
        }
        (Some(bot_id), None) => Connection::Bot(bot_id),
        (None, Some(user_id)) => Connection::User(user_id),
        (None, None) => Connection::Neither,
    }
}

pub fn get_voice_channel_for_user(guild: &Guild, user_id: &UserId) -> Option<ChannelId> {
    guild
        .voice_states
        .get(user_id)
        .and_then(|voice_state| voice_state.channel_id)
}
