use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        gateway::Ready,
        id::GuildId,
        interactions::{application_command::ApplicationCommandOptionType, Interaction},
        prelude::{Activity, VoiceState},
    },
};

use crate::commands::{
    clear::*, leave::*, now_playing::*, pause::*, play::*, playtop::*, queue::*, remove::*,
    repeat::*, resume::*, seek::*, shuffle::*, skip::*, stop::*, summon::*, version::*,
};

pub struct SerenityHandler;

#[async_trait]
impl EventHandler for SerenityHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("🦜 {} is connected!", ready.user.name);

        let activity = Activity::listening("/play");
        ctx.set_activity(activity).await;

        let yellow_flannel = GuildId(79541187794444288);

        // let commands = yellow_flannel
        //     .get_application_commands(&ctx.http)
        //     .await
        //     .unwrap();

        // for command in commands.iter() {
        //     yellow_flannel
        //         .delete_application_command(&ctx.http, command.id)
        //         .await
        //         .unwrap();
        // }

        GuildId::set_application_commands(&yellow_flannel, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("clear").description("Clears the queue")
                })
                .create_application_command(|command| {
                    command
                        .name("leave")
                        .description("Leave the voice channel the bot is connected to")
                })
                .create_application_command(|command| {
                    command
                        .name("np")
                        .description("Displays information about the current track")
                })
                .create_application_command(|command| {
                    command
                        .name("pause")
                        .description("Pauses the current track")
                })
                .create_application_command(|command| {
                    command
                        .name("play")
                        .description("Adds a track to the queue")
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
                        .name("playtop")
                        .description("Places a track on the top of the queue")
                        .create_option(|option| {
                            option
                                .name("query")
                                .description("The media to play")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                })
                .create_application_command(|command| {
                    command.name("queue").description("Shows the queue")
                })
                .create_application_command(|command| {
                    command
                        .name("remove")
                        .description("Removes a track from the queue")
                        .create_option(|option| {
                            option
                                .name("index")
                                .description("Position of the track (0 is currently playing)")
                                .kind(ApplicationCommandOptionType::Integer)
                                .required(true)
                        })
                })
                .create_application_command(|command| {
                    command
                        .name("repeat")
                        .description("Toggles looping for the current track")
                })
                .create_application_command(|command| {
                    command
                        .name("resume")
                        .description("Resumes the current track")
                })
                .create_application_command(|command| {
                    command
                        .name("seek")
                        .description("Seeks current track to the given position")
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
                })
                .create_application_command(|command| {
                    command.name("skip").description("Skips the current track")
                })
                .create_application_command(|command| {
                    command
                        .name("stop")
                        .description("Stops the bot and clears the queue")
                })
                .create_application_command(|command| {
                    command
                        .name("summon")
                        .description("Summons the bot in your voice channel")
                })
                .create_application_command(|command| {
                    command
                        .name("version")
                        .description("Displays the current version")
                })
        })
        .await
        .unwrap();

        // ApplicationCommand::create_global_application_command(&ctx.http, |command| {
        //     command.name("ping").description("pinging ur ass")
        // })
        // .await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(mut command) = interaction {
            match command.data.name.as_str() {
                "clear" => clear(&ctx, &mut command).await,
                "leave" => leave(&ctx, &mut command).await,
                "np" => now_playing(&ctx, &mut command).await,
                "pause" => pause(&ctx, &mut command).await,
                "play" => play(&ctx, &mut command).await,
                "playtop" => playtop(&ctx, &mut command).await,
                "queue" => queue(&ctx, &mut command).await,
                "remove" => remove(&ctx, &mut command).await,
                "repeat" => repeat(&ctx, &mut command).await,
                "resume" => resume(&ctx, &mut command).await,
                "seek" => seek(&ctx, &mut command).await,
                "shuffle" => shuffle(&ctx, &mut command).await,
                "skip" => skip(&ctx, &mut command).await,
                "stop" => stop(&ctx, &mut command).await,
                "summon" => summon(&ctx, &mut command, true).await,
                "version" => version(&ctx, &mut command).await,
                _ => unimplemented!(),
            }
            .unwrap();
        }
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
