use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        gateway::Ready,
        id::GuildId,
        prelude::{OnlineStatus, VoiceState},
    },
};

pub struct SerenityHandler;

#[async_trait]
impl EventHandler for SerenityHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("🦜 {} is connected!", ready.user.name);
        ctx.set_presence(None, OnlineStatus::Online).await;
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        guild: Option<GuildId>,
        _old: Option<VoiceState>,
        new: VoiceState,
    ) {
        if new.user_id == ctx.http.get_current_user().await.unwrap().id && !new.deaf {
            guild
                .unwrap()
                .edit_member(&ctx.http, new.user_id, |n| n.deafen(true))
                .await
                .unwrap();
        }
    }
}
