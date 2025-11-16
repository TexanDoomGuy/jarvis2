use chrono::Duration;
use chrono::Utc;
use log::{debug, error, info, trace};
use serde::Deserialize;
use serenity::all::CommandDataOptionValue;
use serenity::all::CommandOptionType;
use serenity::all::CreateAttachment;
use serenity::all::EditMember;
use serenity::all::Timestamp;
use std::{env, process::Command, process::exit};

use serenity::async_trait;
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

const JARVISPY_LOCATION: &str = "/home/texan/Projects/rust/jarvis/src/jarvis.py";
struct Handler;
fn str_after_n_chars(s: &str, n: usize) -> &str {
    let mut chars = s.chars();
    for _ in 0..n {
        chars.next(); // Advance the iterator by 'n' characters
    }
    chars.as_str() // Get the remaining slice
}

fn asking_jarvis(message: &str) -> bool {
    if message.to_lowercase().starts_with("jarvis") {
        debug!("Potential jarvis detected");
        let message_without_jarvis = str_after_n_chars(message, 6);
        debug!("Message wihout jarvis: {message_without_jarvis}");
        message_without_jarvis.starts_with([',', ':', ';'])
    } else {
        false
    }
}
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        // Actually create the commands
        // I could make helper functions, but I'm lazy
        let ping_command = CreateCommand::new("ping").description("A ping command");

        if let Err(why) =
            serenity::model::application::Command::create_global_command(&ctx.http, ping_command)
                .await
        {
            error!("Cannot create slash command: {why}");
        } else {
            info!("Registered /ping command");
        }
        let gif_command = CreateCommand::new("gif")
            .description("Jarvis, make me a gif")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "text",
                    "Text to be put on the gif",
                )
                .required(true),
            );
        if let Err(why) =
            serenity::model::application::Command::create_global_command(&ctx.http, gif_command)
                .await
        {
            error!("Cannot create slash command: {why}");
        } else {
            info!("Registered /gif command");
        }
        let timeout_command = CreateCommand::new("timeout")
            .description("Jarvis, time him out.")
            .add_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "Who's the evil doer?")
                    .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "length",
                    "Hours to timeout for. (max is 28 days or 672 hours)",
                )
                .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "reason",
                    "Good moderation needs everyone able to know what happened",
                )
                .required(true),
            );
        if let Err(why) =
            serenity::model::application::Command::create_global_command(&ctx.http, timeout_command)
                .await
        {
            error!("Cannot create slash command: {why}");
        } else {
            info!("Registered /timeout command");
        }
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        // Handle what the commands do.

        if let Interaction::Command(command) = &interaction
            && command.data.name.as_str() == "ping"
        {
            let data = CreateInteractionResponseMessage::new().content("Pong!");
            let builder = CreateInteractionResponse::Message(data);

            if let Err(why) = command.create_response(&ctx.http, builder).await {
                error!("Cannot respond to slash command: {why}");
            }
        }

        if let Interaction::Command(interaction) = &interaction {
            let length = interaction
                .data
                .options
                .iter()
                .find(|o| o.name == "length")
                .and_then(|o| match &o.value {
                    CommandDataOptionValue::Integer(s) => Some(s),
                    _ => None,
                })
                .unwrap(); // required option, so unwrap is safe

            let user = interaction
                .data
                .options
                .iter()
                .find(|o| o.name == "user")
                .and_then(|o| match &o.value {
                    CommandDataOptionValue::User(s) => Some(s),
                    _ => None,
                })
                .unwrap(); // required option, so unwrap is safe

            let reason = interaction
                .data
                .options
                .iter()
                .find(|o| o.name == "reason")
                .and_then(|o| match &o.value {
                    CommandDataOptionValue::String(s) => Some(s),
                    _ => None,
                })
                .unwrap(); // required option, so unwrap is safe

            let timeout_duration = std::time::Duration::from_secs(*length as u64 * 3600); // The length is in
            // hours, as a &i64, so we have to convert to u64 secconds.

            let now = Utc::now();
            let until = now + Duration::from_std(timeout_duration).unwrap();
            let timestamp = Timestamp::from(until);

            let builder = EditMember::new()
                .disable_communication_until_datetime(timestamp)
                .audit_log_reason(reason);

            let mut message = format!("timed out <@{user}> for {length} hours");
            // This is assuming it works, because if it doesn't, it changes to an error message.
            if let Err(why) = interaction
                .guild_id
                .unwrap()
                .edit_member(&ctx.http, user, builder)
                .await
            {
                error!("Failed to timeout user: {user}");
                error!("{why:?}");
                message = String::from("Internal error. <@617872594032394242> fix it")
            }

            let data = CreateInteractionResponseMessage::new().content(message);
            let builder = CreateInteractionResponse::Message(data);

            if let Err(why) = interaction.create_response(&ctx.http, builder).await {
                error!("Cannot respond to slash command: {why}");
            }
        }

        if let Interaction::Command(interaction) = &interaction
            && interaction.data.name.as_str() == "gif"
        {
            let text = interaction
                .data
                .options
                .iter()
                .find(|o| o.name == "text")
                .and_then(|o| match &o.value {
                    CommandDataOptionValue::String(s) => Some(s),
                    _ => None,
                })
                .unwrap(); // required option, so unwrap is safe
            info!("Making gif with text {text}");
            // let command_str = format!("{} {}", JARVISPY_LOCATION, text);
            let output = Command::new("python")
                .arg(JARVISPY_LOCATION)
                .arg(text)
                .current_dir(std::env::current_dir().unwrap())
                .output()
                .expect("Failed to run jarvis.py");
            if output.status.success() {
                trace!("Success!")
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!(
                    "Error:\n{}\nWorking dir: {}",
                    stderr,
                    std::env::current_dir().unwrap().display()
                );
            }
            let attachment = CreateAttachment::path("./out.gif").await.unwrap();
            let data = CreateInteractionResponseMessage::new().add_file(attachment);
            let builder = CreateInteractionResponse::Message(data);

            if let Err(why) = interaction.create_response(&ctx.http, builder).await {
                error!("Cannot respond to slash command: {why}");
            }
        }
    }

    async fn message(&self, ctx: Context, msg: serenity::model::channel::Message) {
        // Get the channel object               // All of this is to get the name of the channel
        let msg_channel = msg.channel(&ctx.http).await;
        // There may be an error fetching, so we handle that. Could have used a unwrap_or("Unknown") but I'm use lazy
        let msg_channel = match msg_channel {
            Ok(channel) => match channel {
                // Check if the channel is a dm or not. Not sure who is dming, but they might.
                serenity::model::channel::Channel::Guild(guildchannel) => {
                    format!("#{}", guildchannel.name)
                }
                serenity::model::channel::Channel::Private(_) => {
                    format!("Dm with {user}", user = msg.author.name)
                }
                _ => String::from("Unknown"),
            },
            _ => String::from("Unknown"),
        };
        trace!(
            "{msg_channel} | {author}: {message}",
            author = msg.author.name,
            message = msg.content
        );
        if asking_jarvis(&msg.content)
            && let Err(why) = msg.reply_ping(&ctx.http, "On it.").await
        {
            error!("Error sending message: {why}");
        }
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    token: String,
}

fn setup() -> Config {
    env_logger::init();

    info!("Initializing config!");
    let mut config_path = env::current_exe().expect("Failed to get current executable path");

    // Remove the executable filename to get the directory
    config_path.pop();

    // Append the config file name
    config_path.push("config.toml");
    if config_path.exists() {
        info!("Configuration file found at: {:?}", config_path);
        // You can now proceed to read and parse the config file
    } else {
        error!("Configuration file not found at: {:?}", config_path);
        exit(1)
    }
    let toml_file = std::fs::read_to_string(config_path).unwrap();
    trace!("Oh I'm parsing it. I'm parsing it.");
    toml::from_str(toml_file.as_str()).unwrap()
}

#[tokio::main]
async fn main() {
    let config = setup();
    info!("Config initialised!");
    // Login with a bot token from the environment
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&config.token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    info!("Jarvis should be active!");
    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }
}
