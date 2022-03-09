use std::io::{Error, ErrorKind};

use crate::{
    commands::{
        autopause::*, clear::*, leave::*, now_playing::*, pause::*, play::*, queue::*, remove::*,
        repeat::*, resume::*, seek::*, shuffle::*, skip::*, stop::*, summon::*, version::*,
        voteskip::*,
    },
    sources::spotify::Spotify,
    utils::{create_response},
    connection::{check_voice_connections, Connection},
    errors::ParrotError,
};
use lazy_static::lazy_static;
use rspotify::{ClientCredsSpotify, ClientError};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        gateway::Ready,
        guild::Role,
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommand, ApplicationCommandInteraction, ApplicationCommandOptionType,
                ApplicationCommandPermissionType,
            },
            Interaction,
        },
        prelude::{Activity, VoiceState},
    },
    prelude::{Mentionable, SerenityError},
};
use tokio::sync::Mutex;

lazy_static! {
    pub static ref SPOTIFY: Mutex<Result<ClientCredsSpotify, ParrotError>> = Mutex::new(Err(ParrotError::Other("no auth attempts")));
}

pub struct SerenityHandler;

#[async_trait]
impl EventHandler for SerenityHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("🦜 {} is connected!", ready.user.name);

        // sets parrot activity status message to /play
        let activity = Activity::listening("/play");
        ctx.set_activity(activity).await;

        // attempt to authenticate to spotify
        *SPOTIFY.lock().await = Spotify::auth().await;

        // creates the global application commands
        // and sets them with the correct permissions
        self.set_commands(&ctx, ready).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(mut command) = interaction {
            if let Err(err) = self.run_command(&ctx, &mut command).await {
                self.handle_error(&ctx, &mut command, err).await
            }
        }
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        guild: Option<GuildId>,
        _old: Option<VoiceState>,
        new: VoiceState,
    ) {
        self.self_deafen(&ctx, guild, new).await;
    }
}

impl SerenityHandler {
    async fn apply_role(
        &self,
        ctx: &Context,
        role: Role,
        guild: GuildId,
        commands: &[ApplicationCommand],
    ) {
        let commands = commands
            .iter()
            .filter(|command| !command.default_permission);
        for command in commands {
            guild
                .create_application_command_permission(&ctx.http, command.id, |p| {
                    p.create_permission(|d| {
                        d.kind(ApplicationCommandPermissionType::Role)
                            .id(role.id.0)
                            .permission(true)
                    })
                })
                .await
                .expect("failed to create command permission");
        }
    }

    async fn create_commands(&self, ctx: &Context) -> Vec<ApplicationCommand> {
        ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command
                        .name("autopause")
                        .description("Toggles whether to pause after a song ends")
                        .default_permission(false)
                })
                .create_application_command(|command| {
                    command.name("clear").description("Clears the queue")
                    .default_permission(false)
                })
                .create_application_command(|command| {
                    command
                        .name("leave")
                        .description("Leave the voice channel the bot is connected to")
                        .default_permission(false)
                })
                .create_application_command(|command| {
                    command
                        .name("np")
                        .description("Displays information about the current track")
                        .default_permission(true)
                })
                .create_application_command(|command| {
                    command
                        .name("pause")
                        .description("Pauses the current track")
                        .default_permission(false)
                })
                .create_application_command(|command| {
                    command
                        .name("play")
                        .description("Add a track to the queue")
                        .default_permission(true)
                        .create_option(|option| {
                                option
                                    .name("query")
                                    .description("The media to play")
                                    .kind(ApplicationCommandOptionType::String)
                                    .required(true)
                        })
                })
                .create_application_command(|command| {
                    command
                        .name("superplay")
                        .description("Add a track to the queue in a special way")
                        .default_permission(false)
                        .create_option(|option| {
                            option
                                .name("next")
                                .description("Add a track to be played up next")
                                .kind(ApplicationCommandOptionType::SubCommand)
                                .create_sub_option(|option| {
                                    option
                                        .name("query")
                                        .description("The media to play")
                                        .kind(ApplicationCommandOptionType::String)
                                        .required(true)
                                })
                        })
                        .create_option(|option| {
                            option
                                .name("jump")
                                .description("Instantly plays a track, skipping the current one")
                                .kind(ApplicationCommandOptionType::SubCommand)
                                .create_sub_option(|option| {
                                    option.name("query")
                                    .description("The media to play")
                                    .kind(ApplicationCommandOptionType::String)
                                    .required(true)
                                })
                        })
                        .create_option(|option| {
                            option
                                .name("all")
                                .description("Add all tracks if the URL refers to a video and a playlist")
                                .kind(ApplicationCommandOptionType::SubCommand)
                                .create_sub_option(|option| {
                                    option
                                        .name("query")
                                        .description("The media to play")
                                        .kind(ApplicationCommandOptionType::String)
                                        .required(true)
                                })
                        })
                        .create_option(|option| {
                            option
                                .name("reverse")
                                .description("Add a playlist to the queue in reverse order")
                                .kind(ApplicationCommandOptionType::SubCommand)
                                .create_sub_option(|option| {
                                    option
                                        .name("query")
                                        .description("The media to play")
                                        .kind(ApplicationCommandOptionType::String)
                                        .required(true)
                                })
                        })
                        .create_option(|option| {
                            option
                                .name("shuffle")
                                .description("Add a playlist to the queue in random order")
                                .kind(ApplicationCommandOptionType::SubCommand)
                                .create_sub_option(|option| {
                                    option
                                        .name("query")
                                        .description("The media to play")
                                        .kind(ApplicationCommandOptionType::String)
                                        .required(true)
                                })
                        })
                })
                .create_application_command(|command| {
                    command.name("queue").description("Shows the queue").default_permission(true)
                })
                .create_application_command(|command| {
                    command
                        .name("remove")
                        .description("Removes a track from the queue")
                        .default_permission(false)
                        .create_option(|option| {
                            option
                                .name("index")
                                .description("Position of the track in the queue (1 is the next track to be played)")
                                .kind(ApplicationCommandOptionType::Integer)
                                .required(true)
                                .min_int_value(1)
                        })
                        .create_option(|option| {
                            option
                                .name("until")
                                .description("Upper range track position to remove a range of tracks")
                                .kind(ApplicationCommandOptionType::Integer)
                                .required(false)
                                .min_int_value(1)
                        })
                })
                .create_application_command(|command| {
                    command
                        .name("repeat")
                        .description("Toggles looping for the current track")
                        .default_permission(false)
                })
                .create_application_command(|command| {
                    command
                        .name("resume")
                        .description("Resumes the current track")
                        .default_permission(false)
                })
                .create_application_command(|command| {
                    command
                        .name("seek")
                        .description("Seeks current track to the given position")
                        .default_permission(false)
                        .create_option(|option| {
                            option
                                .name("timestamp")
                                .description("Timestamp in the format HH:MM:SS")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                })
                .create_application_command(|command| {
                    command.name("shuffle").description("Shuffles the queue")
                    .default_permission(false)
                })
                .create_application_command(|command| {
                    command.name("skip").description("Skips the current track")
                    .default_permission(false)
                    .create_option(|option| {
                        option
                            .name("to")
                            .description("Track index to skip to")
                            .kind(ApplicationCommandOptionType::Integer)
                            .required(false)
                            .min_int_value(1)
                    })
                })
                .create_application_command(|command| {
                    command
                        .name("stop")
                        .description("Stops the bot and clears the queue")
                        .default_permission(false)
                })
                .create_application_command(|command| {
                    command
                        .name("summon")
                        .description("Summons the bot in your voice channel")
                        .default_permission(true)
                })
                .create_application_command(|command| {
                    command
                        .name("version")
                        .description("Displays the current version")
                        .default_permission(true)
                })
                .create_application_command(|command| {
                    command.name("voteskip").description("Starts a vote to skip the current track")
                    .default_permission(true)
                })
        })
        .await
        .expect("failed to create command")
    }

    async fn ensure_role(
        &self,
        ctx: &Context,
        guild: GuildId,
        role_name: &str,
    ) -> Result<Role, SerenityError> {
        let roles = guild.roles(&ctx.http).await?;
        let role = roles.iter().find(|(_, role)| role.name == role_name);
        match role {
            Some((_, role)) => Ok(role.to_owned()),
            None => {
                guild
                    .create_role(&ctx.http, |r| r.name(role_name).mentionable(true))
                    .await
            }
        }
    }

    async fn run_command(
        &self,
        ctx: &Context,
        command: &mut ApplicationCommandInteraction,
    ) -> Result<(), ParrotError> {
        let command_name = command.data.name.as_str();

        let guild_id = command.guild_id.unwrap();
        let guild = ctx.cache.guild(guild_id).await.unwrap();

        // get songbird voice client
        let manager = songbird::get(ctx).await.unwrap();

        // parrot might have been disconnected manually
        if let Some(call) = manager.get(guild.id) {
            let mut handler = call.lock().await;
            if handler.current_connection().is_none() {
                handler.leave().await.unwrap();
            }
        }

        // fetch the user and the bot's user IDs
        let user_id = command.user.id;
        let bot_id = ctx.cache.current_user_id().await;

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
            "np" => now_playing(ctx, command).await,
            "pause" => pause(ctx, command).await,
            "play" => play(ctx, command).await,
            "queue" => queue(ctx, command).await,
            "remove" => remove(ctx, command).await,
            "repeat" => repeat(ctx, command).await,
            "resume" => resume(ctx, command).await,
            "seek" => seek(ctx, command).await,
            "shuffle" => shuffle(ctx, command).await,
            "skip" => skip(ctx, command).await,
            "superplay" => play(ctx, command).await,
            "stop" => stop(ctx, command).await,
            "summon" => summon(ctx, command, true).await,
            "version" => version(ctx, command).await,
            "voteskip" => voteskip(ctx, command).await,
            _ => unreachable!(),
        }
    }

    async fn self_deafen(&self, ctx: &Context, guild: Option<GuildId>, new: VoiceState) {
        if new.user_id == ctx.http.get_current_user().await.unwrap().id && !new.deaf {
            guild
                .unwrap()
                .edit_member(&ctx.http, new.user_id, |n| n.deafen(true))
                .await
                .unwrap();
        }
    }

    async fn set_commands(&self, ctx: &Context, ready: Ready) {
        let commands = self.create_commands(ctx).await;
        let role_name = ready.user.name + "'s DJ";
        for guild in ready.guilds {
            let guild_id = guild.id();

            // ensures the role exists, creating it if does not
            // if it fails to create the role (e.g. no permissions)
            // it does nothing but output a debug log
            match self.ensure_role(ctx, guild_id, &role_name).await {
                Ok(role) => self.apply_role(ctx, role, guild_id, &commands).await,
                Err(err) => println!(
                    "Could not create '{role_name}' role for guild {guild_id} because {err:?}"
                ),
            };
        }
    }

    async fn handle_error(
        &self,
        ctx: &Context,
        interaction: &mut ApplicationCommandInteraction,
        err: ParrotError,
    ) {
        create_response(&ctx.http, interaction, &format!("{err}"))
            .await
            .expect("failed to create response");
    }
}
