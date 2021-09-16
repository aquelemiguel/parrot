use serenity::{builder::CreateEmbed, utils::Colour};
use std::time::Duration;

pub fn create_default_embed(embed: &mut CreateEmbed, title: &str, description: &str) -> () {
    embed.title(format!("{}", title));
    embed.description(description);
    embed.colour(Colour::ORANGE);
}

pub fn get_human_readable_timestamp(duration: Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = duration.as_secs() / 3600;

    let timestamp = if hours < 1 {
        format!("{}:{:02}", minutes, seconds)
    } else {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    };

    timestamp
}
