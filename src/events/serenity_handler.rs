use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        gateway::Ready,
        id::GuildId,
        interactions::{
            application_command::ApplicationCommand, Interaction, InteractionResponseType,
        },
        prelude::{Activity, VoiceState},
    },
};

use crate::commands::{summon::*, version::*};

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

        let commands = yellow_flannel
            .get_application_commands(&ctx.http)
            .await
            .unwrap();

        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );

        // ApplicationCommand::create_global_application_command(&ctx.http, |command| {
        //     command.name("ping").description("pinging ur ass")
        // })
        // .await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(mut command) = interaction {
            match command.data.name.as_str() {
                "summon" => summon(&ctx, &mut command).await,
                "version" => version(&ctx, &mut command).await,
                _ => unimplemented!(),
            };
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
