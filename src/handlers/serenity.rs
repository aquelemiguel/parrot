use crate::{
    commands::{
        autopause::*, clear::*, leave::*, manage_sources::*, now_playing::*, pause::*, play::*,
        queue::*, remove::*, repeat::*, resume::*, seek::*, shuffle::*, skip::*, stop::*,
        summon::*, version::*, voteskip::*,
    },
    connection::{check_voice_connections, Connection},
    errors::ParrotError,
    guild::settings::{GuildSettings, GuildSettingsMap},
    handlers::track_end::update_queue_messages,
    sources::spotify::{Spotify, SPOTIFY},
    utils::create_response_text,
};
use serenity::{
    all::{CommandOptionType, CreateCommand, CreateCommandOption, EditMember},
    async_trait,
    client::{Context, EventHandler},
    gateway::ActivityData,
    model::{
        application::Command, application::Interaction, gateway::Ready, id::GuildId,
        voice::VoiceState,
    },
    prelude::Mentionable,
};

pub struct SerenityHandler;

#[async_trait]
impl EventHandler for SerenityHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("🦜 {} is connected!", ready.user.name);

        // sets parrot activity status message to /play
        let activity = ActivityData::listening("/play");
        ctx.set_activity(Some(activity));

        // attempts to authenticate to spotify
        *SPOTIFY.lock().await = Spotify::auth().await;

        // creates the global application commands
        self.create_commands(&ctx).await;

        // loads serialized guild settings
        self.load_guilds_settings(&ctx, &ready).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(mut command) = interaction else {
            return;
        };

        if let Err(err) = self.run_command(&ctx, &mut command).await {
            self.handle_error(&ctx, &mut command, err).await
        }
    }

    async fn voice_state_update(&self, ctx: Context, _old: Option<VoiceState>, new: VoiceState) {
        // do nothing if this is a voice update event for a user, not a bot
        if new.user_id != ctx.cache.current_user().id {
            return;
        }

        if new.channel_id.is_some() {
            return self.self_deafen(&ctx, new.guild_id, new).await;
        }

        let Some(manager) = songbird::get(&ctx).await else {
            return;
        };
        let Some(guild_id) = new.guild_id else {
            return;
        };

        if manager.get(guild_id).is_some() {
            manager.remove(guild_id).await.ok();
        }

        update_queue_messages(&ctx.http, &ctx.data, &[], guild_id).await;
    }
}

impl SerenityHandler {
    async fn create_commands(&self, ctx: &Context) -> Vec<Command> {
        let commands = vec![
            CreateCommand::new("autopause")
                .description("Toggles whether to pause after a song ends"),
            CreateCommand::new("clear").description("Clears the queue"),
            CreateCommand::new("leave")
                .description("Leave the voice channel the bot is connected to"),
            CreateCommand::new("managesources")
                .description("Manage streaming from different sources"),
            CreateCommand::new("np").description("Displays information about the current track"),
            CreateCommand::new("pause").description("Pauses the current track"),
            CreateCommand::new("play")
                .description("Add a track to the queue")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "query",
                        "The media to play",
                    )
                    .required(true),
                ),
            CreateCommand::new("superplay")
                .description("Add a track to the queue in a special way")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "next",
                        "Add a track to be played up next",
                    )
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "query",
                            "The media to play",
                        )
                        .required(true),
                    ),
                )
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "jump",
                        "Instantly plays a track, skipping the current one",
                    )
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "query",
                            "The media to play",
                        )
                        .required(true),
                    ),
                )
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "all",
                        "Add all tracks if the URL refers to a video and a playlist",
                    )
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "query",
                            "The media to play",
                        )
                        .required(true),
                    ),
                )
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "reverse",
                        "Add a playlist to the queue in reverse order",
                    )
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "query",
                            "The media to play",
                        )
                        .required(true),
                    ),
                )
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "shuffle",
                        "Add a playlist to the queue in random order",
                    )
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "query",
                            "The media to play",
                        )
                        .required(true),
                    ),
                ),
            CreateCommand::new("queue").description("Shows the queue"),
            CreateCommand::new("remove")
                .description("Removes a track from the queue")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::Integer,
                        "index",
                        "Position of the track in the queue (1 is the next track to be played)",
                    )
                    .required(true)
                    .min_int_value(1),
                )
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::Integer,
                        "until",
                        "Upper range track position to remove a range of tracks",
                    )
                    .required(false)
                    .min_int_value(1),
                ),
            CreateCommand::new("repeat").description("Toggles looping for the current track"),
            CreateCommand::new("resume").description("Resumes the current track"),
            CreateCommand::new("seek")
                .description("Seeks current track to the given position")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "timestamp",
                        "Timestamp in the format HH:MM:SS",
                    )
                    .required(true),
                ),
            CreateCommand::new("shuffle").description("Shuffles the queue"),
            CreateCommand::new("skip")
                .description("Skips the current track")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::Integer,
                        "to",
                        "Track index to skip to",
                    )
                    .required(false)
                    .min_int_value(1),
                ),
            CreateCommand::new("stop").description("Stops the bot and clears the queue"),
            CreateCommand::new("summon").description("Summons the bot in your voice channel"),
            CreateCommand::new("version").description("Displays the current version"),
            CreateCommand::new("voteskip").description("Starts a vote to skip the current track"),
        ];

        Command::set_global_commands(&ctx.http, commands)
            .await
            .expect("failed to create command")
    }

    async fn load_guilds_settings(&self, ctx: &Context, ready: &Ready) {
        println!("[INFO] Loading guilds' settings");
        let mut data = ctx.data.write().await;
        for guild in &ready.guilds {
            println!("[DEBUG] Loading guild settings for {:?}", guild);
            let settings = data.get_mut::<GuildSettingsMap>().unwrap();

            let guild_settings = settings
                .entry(guild.id)
                .or_insert_with(|| GuildSettings::new(guild.id));

            if let Err(err) = guild_settings.load_if_exists() {
                println!(
                    "[ERROR] Failed to load guild {} settings due to {}",
                    guild.id, err
                );
            }
        }
    }

    async fn run_command(
        &self,
        ctx: &Context,
        command: &mut serenity::all::CommandInteraction,
    ) -> Result<(), ParrotError> {
        let command_name = command.data.name.as_str();

        let guild_id = command.guild_id.ok_or(ParrotError::Other(
            "This command can only be used in a server",
        ))?;

        // Clone the guild to avoid holding CacheRef across await points
        let guild = ctx
            .cache
            .guild(guild_id)
            .ok_or(ParrotError::Other("Guild not found in cache"))?
            .clone();

        // get songbird voice client
        let manager = songbird::get(ctx)
            .await
            .ok_or(ParrotError::Other("Voice manager not configured"))?;

        // parrot might have been disconnected manually
        if let Some(call) = manager.get(guild.id) {
            let mut handler = call.lock().await;
            if handler.current_connection().is_none() {
                handler.leave().await.ok();
            }
        }

        // fetch the user and the bot's user IDs
        let user_id = command.user.id;
        let bot_id = ctx.cache.current_user().id;

        match command_name {
            "autopause" | "clear" | "leave" | "pause" | "remove" | "repeat" | "resume" | "seek"
            | "shuffle" | "skip" | "stop" | "voteskip" => {
                match check_voice_connections(&guild, &user_id, &bot_id) {
                    Connection::User(_) | Connection::Neither => Err(ParrotError::NotConnected),
                    Connection::Bot(bot_channel_id) => {
                        Err(ParrotError::AuthorDisconnected(bot_channel_id.mention()))
                    }
                    Connection::Separate(_, _) => Err(ParrotError::WrongVoiceChannel),
                    _ => Ok(()),
                }
            }
            "play" | "superplay" | "summon" => {
                match check_voice_connections(&guild, &user_id, &bot_id) {
                    Connection::User(_) => Ok(()),
                    Connection::Bot(_) if command_name == "summon" => {
                        Err(ParrotError::AuthorNotFound)
                    }
                    Connection::Bot(_) if command_name != "summon" => {
                        Err(ParrotError::WrongVoiceChannel)
                    }
                    Connection::Separate(bot_channel_id, _) => {
                        Err(ParrotError::AlreadyConnected(bot_channel_id.mention()))
                    }
                    Connection::Neither => Err(ParrotError::AuthorNotFound),
                    _ => Ok(()),
                }
            }
            "np" | "queue" => match check_voice_connections(&guild, &user_id, &bot_id) {
                Connection::User(_) | Connection::Neither => Err(ParrotError::NotConnected),
                _ => Ok(()),
            },
            _ => Ok(()),
        }?;

        match command_name {
            "autopause" => autopause(ctx, command).await,
            "clear" => clear(ctx, command).await,
            "leave" => leave(ctx, command).await,
            "managesources" => allow(ctx, command).await,
            "np" => now_playing(ctx, command).await,
            "pause" => pause(ctx, command).await,
            "play" | "superplay" => play(ctx, command).await,
            "queue" => queue(ctx, command).await,
            "remove" => remove(ctx, command).await,
            "repeat" => repeat(ctx, command).await,
            "resume" => resume(ctx, command).await,
            "seek" => seek(ctx, command).await,
            "shuffle" => shuffle(ctx, command).await,
            "skip" => skip(ctx, command).await,
            "stop" => stop(ctx, command).await,
            "summon" => summon(ctx, command, true).await,
            "version" => version(ctx, command).await,
            "voteskip" => voteskip(ctx, command).await,
            _ => unreachable!(),
        }
    }

    async fn self_deafen(&self, ctx: &Context, guild: Option<GuildId>, new: VoiceState) {
        let Ok(user) = ctx.http.get_current_user().await else {
            return;
        };

        if user.id == new.user_id && !new.deaf {
            if let Some(guild_id) = guild {
                if let Err(e) = guild_id
                    .edit_member(&ctx.http, new.user_id, EditMember::new().deafen(true))
                    .await
                {
                    eprintln!("[WARN] Failed to self-deafen: {}", e);
                }
            }
        }
    }

    async fn handle_error(
        &self,
        ctx: &Context,
        interaction: &mut serenity::all::CommandInteraction,
        err: ParrotError,
    ) {
        create_response_text(&ctx.http, interaction, &format!("{err}"))
            .await
            .expect("failed to create response");
    }
}
