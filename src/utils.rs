use serenity::{
    client::Context,
    http::Http,
    model::{channel::Message, id::RoleId, prelude::User},
    utils::Color,
};
use std::{sync::Arc, time::Duration};

pub async fn send_simple_message(http: &Arc<Http>, msg: &Message, content: &str) {
    msg.channel_id
        .send_message(http, |m| {
            m.embed(|e| e.description(format!("**{}**", content)).color(Color::RED))
        })
        .await
        .expect("Unable to send message");
}

pub fn get_human_readable_timestamp(duration: Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = duration.as_secs() / 3600;

    if hours < 1 {
        format!("{}:{:02}", minutes, seconds)
    } else {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    }
}

pub fn get_full_username(user: &User) -> String {
    format!("{}#{:04}", user.name, user.discriminator)
}

pub async fn author_is_dj(ctx: &Context, msg: &Message) -> bool {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let mut roleid = RoleId::default();

    for (_, role) in guild.roles {
        if role.name == "DJ" {
            roleid = role.id;
        }
    }

    let user = &msg.author;
    user.has_role(&ctx.http, guild.id, roleid).await.unwrap()
}
