#![feature(async_closure)]
//! Requires the 'framework' feature flag be enabled in your project's
//! `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["framework", "standard_framework"]
//! ```
use std::{collections::{HashMap, HashSet}, env, fmt::Write, sync::Arc};
use serenity::{
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        Args, CommandOptions, CommandResult, CheckResult, CommandGroup,
        DispatchError, HelpOptions, help_commands, StandardFramework,
        macros::{command, group, help, check},
    },
    model::{channel::{Channel, Message}, gateway::Ready, id::UserId},
    utils::{content_safe, ContentSafeOptions, Mutex},
};
use async_trait::async_trait;

// This imports `typemap`'s `Key` as `TypeMapKey`.
use serenity::prelude::*;

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(about, say, commands, ping, some_long_command)]
struct General;

#[group]
// Sets multiple prefixes for a group.
// This requires us to call commands in this group
// via `~emoji` (or `~em`) instead of just `~`.
#[prefixes("emoji", "em")]
// Set a description to appear if a user wants to display a single group
// e.g. via help using the group-name or one of its prefixes.
#[description = "A group with commands providing an emoji as response."]
// Sets a command that will be executed if only a group-prefix was passed.
#[default_command(bird)]
#[commands(cat, dog)]
struct Emoji;

#[group]
// Sets a single prefix for this group.
// So one has to call commands in this group
// via `~math` instead of just `~`.
#[prefix = "math"]
#[commands(multiply)]
struct Math;

#[group]
#[owners_only]
// Limit all commands to be guild-restricted.
#[only_in(guilds)]
// Adds checks that need to be passed.
#[checks(Admin)]
#[commands(am_i_admin, slow_mode)]
struct Owner;

// The framework provides two built-in help commands for you to use.
// But you can also make your own customized help command that forwards
// to the behaviour of either of them.
#[help]
// This replaces the information that a user can pass
// a command-name as argument to gain specific information about it.
#[individual_command_tip =
"Hello! こんにちは！Hola! Bonjour! 您好!\n\
If you want more information about a specific command, just pass the command as argument."]
// Some arguments require a `{}` in order to replace it with contextual information.
// In this case our `{}` refers to a command's name.
#[command_not_found_text = "Could not find: `{}`."]
// Define the maximum Levenshtein-distance between a searched command-name
// and commands. If the distance is lower than or equal the set distance,
// it will be displayed as a suggestion.
// Setting the distance to 0 will disable suggestions.
#[max_levenshtein_distance(3)]
// When you use sub-groups, Serenity will use the `indention_prefix` to indicate
// how deeply an item is indented.
// The default value is "-", it will be changed to "+".
#[indention_prefix = "+"]
// On another note, you can set up the help-menu-filter-behaviour.
// Here are all possible settings shown on all possible options.
// First case is if a user lacks permissions for a command, we can hide the command.
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
#[lacking_role = "Nothing"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible
// cases of ~~strikethrough-commands~~, but only if
// `strikethrough_commands_tip(Some(""))` keeps `Some()` wrapping an empty `String`, which is the default value.
// If the `String` is not empty, your given `String` will be used instead.
// If you pass in a `None`, no hint will be displayed at all.
async fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await
}

//struct ClientHandler {}

/*
#[async_trait]
impl BeforeHandler for ClientHandler {
    async fn before_handler(&self, ctx : &mut Context, msg : &Message, command_name : &str) -> bool {
        println!("Got command '{}' by user '{}'",
                 command_name,
                 msg.author.name);

        // Increment the number of times this command has been run once. If
        // the command's name does not exist in the counter, add a default
        // value of 0.
        let mut data = ctx.data.write().await;
        let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in ShareMap.");
        let entry = counter.entry(command_name.to_string()).or_insert(0);
        *entry += 1;

        true // if `before` returns false, command processing doesn't happen.
    }
}

#[async_trait]
impl AfterHandler for ClientHandler {
    async fn after_handler(&self, _ctx: &mut Context, _msg: &Message, command_name: &str, error: Result<(), CommandError>) {
        match error {
            Ok(()) => println!("Processed command '{}'", command_name),
            Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
        }
    }
}
*/

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect(
        "Expected a token in the environment",
    );
    let mut client = Client::new(&token, Handler).await.expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match client.cache_and_http.http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };


    // Commands are equivalent to:
    // "~about"
    // "~emoji cat"
    // "~emoji dog"
    // "~multiply"
    // "~ping"
    // "~some long command"
    client.with_framework(
        // Configures the client, allowing for options to mutate how the
        // framework functions.
        //
        // Refer to the documentation for
        // `serenity::ext::framework::Configuration` for all available
        // configurations.
        StandardFramework::new()
        .configure(|c| c
            .with_whitespace(true)
            .on_mention(Some(bot_id))
            .prefix("~")
            // You can set multiple delimiters via delimiters()
            // or just one via delimiter(",")
            // If you set multiple delimiters, the order you list them
            // decides their priority (from first to last).
            //
            // In this case, if "," would be first, a message would never
            // be delimited at ", ", forcing you to trim your arguments if you
            // want to avoid whitespaces at the start of each.
            .delimiters(vec![", ", ","])
            // Sets the bot's owners. These will be used for commands that
            // are owners only.
            .owners(owners))

        // Set a function to be called prior to each command execution. This
        // provides the context of the command, the message that was received,
        // and the full name of the command that will be called.
        //
        // You can not use this to determine whether a command should be
        // executed. Instead, the `#[check]` macro gives you this functionality.
        .before(|_ctx, msg, command_name| {
            println!("Got command '{}' by user '{}'",
                     command_name,
                     msg.author.name);

            // Increment the number of times this command has been run once. If
            // the command's name does not exist in the counter, add a default
            // value of 0.
            /* TODO: async closure or something here
            let mut data = ctx.data.write().await;
            let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in ShareMap.");
            let entry = counter.entry(command_name.to_string()).or_insert(0);
            *entry += 1;
            */

            true // if `before` returns false, command processing doesn't happen.
        })
        // Similar to `before`, except will be called directly _after_
        // command execution.
        .after(|_, _, command_name, error| {
            match error {
                Ok(()) => println!("Processed command '{}'", command_name),
                Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
            }
        })
        // Set a function that's called whenever an attempted command-call's
        // command could not be found.
        .unrecognised_command(|_, _, unknown_command_name| {
            println!("Could not find command named '{}'", unknown_command_name);
        })
        // Set a function that's called whenever a message is not a command.
        .normal_message(|_, message| {
            println!("Message is not a command '{}'", message.content);
        })
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        .on_dispatch_error(|ctx, msg, error| {
            if let DispatchError::Ratelimited(seconds) = error {
                let _ = msg.channel_id.say(&ctx.http, &format!("Try this again in {} seconds.", seconds));
            }
        })
        .help(&MY_HELP)
        // Can't be used more than once per 5 seconds:
        .bucket("emoji", |b| b.delay(5))
        // Can't be used more than 2 times per 30 seconds, with a 5 second delay:
        .bucket("complicated", |b| b.delay(5).time_span(30).limit(2))
        // The `#[group]` macro generates `static` instances of the options set for the group.
        // They're made in the pattern: `#name_GROUP` for the group instance and `#name_GROUP_OPTIONS`.
        // #name is turned all uppercase
        .group(&GENERAL_GROUP)
        .group(&EMOJI_GROUP)
        .group(&MATH_GROUP)
        .group(&OWNER_GROUP)
    ).await;

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

// Commands can be created via the attribute `#[command]` macro.
#[command]
// Options are passed via subsequent attributes.
// Make this command use the "complicated" bucket.
#[bucket = "complicated"]
async fn commands(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mut contents = "Commands used:\n".to_string();

    {
        let data = ctx.data.read().await;
        let counter = data.get::<CommandCounter>().expect("Expected CommandCounter in ShareMap.");

        for (k, v) in counter {
            let _ = write!(contents, "- {name}: {amount}\n", name = k, amount = v);
        }

        if let Err(why) = msg.channel_id.say(&ctx.http, &contents).await {
            println!("Error sending message: {:?}", why);
        }
    }

    Ok(())
}

// Repeats what the user passed as argument but ensures that user and role
// mentions are replaced with a safe textual alternative.
// In this example channel mentions are excluded via the `ContentSafeOptions`.
#[command]
async fn say(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let settings = if let Some(guild_id) = msg.guild_id {
        // By default roles, users, and channel mentions are cleaned.
        ContentSafeOptions::default()
            // We do not want to clean channal mentions as they
            // do not ping users.
            .clean_channel(false)
            // If it's a guild channel, we want mentioned users to be displayed
            // as their display name.
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(false)
    };

    let rest = &args.rest();
    let content = content_safe(&ctx.cache, rest, &settings).await;

    if let Err(why) = msg.channel_id.say(&ctx.http, &content).await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

// A function which acts as a "check", to determine whether to call a command.
//
// In this case, this command checks to ensure you are the owner of the message
// in order for the command to be executed. If the check fails, the command is
// not called.
#[check]
#[name = "Owner"]
async fn owner_check(_ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
        // Replace 7 with your ID to make this check pass.
        //
        // `true` will convert into `CheckResult::Success`,
        //
        // `false` will convert into `CheckResult::Failure(Reason::Unknown)`,
        //
        // and if you want to pass a reason alongside failure you can do:
        // `CheckResult::new_user("Lacked admin permission.")`,
        //
        // if you want to mark it as something you want to log only:
        // `CheckResult::new_log("User lacked admin permission.")`,
        //
        // and if the check's failure origin is unknown you can mark it as such (same as using `false.into`):
        // `CheckResult::new_unknown()`
        let res = msg.author.id == 7;
        res.into()
}

// A function which acts as a "check", to determine whether to call a command.
//
// This check analyses whether a guild member permissions has
// administrator-permissions.
#[check]
#[name = "Admin"]
// Whether the check shall be tested in the help-system.
#[check_in_help(true)]
// Whether the check shall be displayed in the help-system.
#[display_in_help(true)]
async fn admin_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    if let Some(member) = msg.member(&ctx.cache).await {
        if let Ok(permissions) = member.permissions(&ctx.cache).await {
            return permissions.administrator().into();
        }
    }

    false.into()
}

#[command]
async fn some_long_command(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let res = format!("Arguments: {:?}", args.rest());
    if let Err(why) = msg.channel_id.say(&ctx.http, &res).await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

#[command]
// Limits the usage of this command to roles named:
#[allowed_roles("mods", "ultimate neko")]
async fn about_role(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let potential_role_name = args.rest();

    if let Some(guild) = msg.guild(&ctx.cache).await {
        // `role_by_name()` allows us to attempt attaining a reference to a role
        // via its name.
        if let Some(role) = guild.read().await.role_by_name(&potential_role_name) {
            let res = format!("Role-ID: {}", role.id);
            if let Err(why) = msg.channel_id.say(&ctx.http, &res).await {
                println!("Error sending message: {:?}", why);
            }

            return Ok(());
        }
    }

    if let Err(why) = msg.channel_id.say(&ctx.http, format!("Could not find role named: {:?}", potential_role_name)).await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

#[command]
// Lets us also call `~math *` instead of just `~math multiply`.
#[aliases("*")]
async fn multiply(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
        let first = args.single::<f64>();
        let second = args.single::<f64>();

        if let (Ok(first), Ok(second)) = (first, second) {
            let res = (first * second).to_string();

            if let Err(why) = msg.channel_id.say(&ctx.http, &res).await {
                println!("Err sending product of {} and {}: {:?}", first, second, why);
            }
        } else {
            println!("Err computing product");
        }

    Ok(())
}

#[command]
async fn about(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Err(why) = msg.channel_id.say(&ctx.http, "This is a small test-bot! : )").await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

#[command]
async fn latency(ctx: &mut Context, msg: &Message) -> CommandResult {
    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    {
        let data = ctx.data.read().await;

        let shard_manager = match data.get::<ShardManagerContainer>() {
            Some(v) => Some(v),
            None => {
                let _ = msg.reply(&ctx, "There was a problem getting the shard manager").await;
                None
            },
        };

        if let Some(shard_manager) = shard_manager {
            let manager = shard_manager.lock().await;

            // Shards are backed by a "shard runner" responsible for processing events
            // over the shard, so we'll get the information about the shard runner for
            // the shard this command was sent over.
            let runner = match manager.runners.async_get(ShardId(ctx.shard_id)).await {
                Some(runner) => Some(runner),
                None => {
                    let _ = msg.reply(&ctx, "No shard found").await;
                    None
                },
            };

            if let Some(runner) = runner {
                let content = format!("The shard latency is {:?}", runner.latency);
                let _ = msg.reply(&ctx, &content).await;
            }
        }
    }

    Ok(())
}

#[command]
// Limit command usage to guilds.
#[only_in(guilds)]
#[checks(Owner)]
async fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Err(why) = msg.channel_id.say(&ctx.http, "Pong! : )").await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

#[command]
// Adds multiple aliases
#[aliases("kitty", "neko")]
// Make this command use the "emoji" bucket.
#[bucket = "emoji"]
// Allow only administrators to call this:
#[required_permissions("ADMINISTRATOR")]
async fn cat(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Err(why) = msg.channel_id.say(&ctx.http, ":cat:").await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

#[command]
#[description = "Sends an emoji with a dog."]
#[bucket = "emoji"]
async fn dog(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Err(why) = msg.channel_id.say(&ctx.http, ":dog:").await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

#[command]
async fn bird(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let say_content = if args.is_empty() {
        ":bird: can find animals for you.".to_string()
    } else {
        format!(":bird: could not find animal named: `{}`.", args.rest())
    };

    if let Err(why) = msg.channel_id.say(&ctx.http, say_content).await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

#[command]
async fn am_i_admin(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Err(why) = msg.channel_id.say(&ctx.http, "Yes you are.").await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}


#[command]
async fn slow_mode(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let say_content = if let Ok(slow_mode_rate_seconds) = args.single::<u64>() {
        if let Err(why) = msg.channel_id.edit(&ctx.http, |c| c.slow_mode_rate(slow_mode_rate_seconds)).await {
            println!("Error setting channel's slow mode rate: {:?}", why);

            format!("Failed to set slow mode to `{}` seconds.", slow_mode_rate_seconds)
        } else {
            format!("Successfully set slow mode rate to `{}` seconds.", slow_mode_rate_seconds)
        }
    } else if let Some(Channel::Guild(channel)) = msg.channel_id.to_channel_cached(&ctx.cache).await {
        format!("Current slow mode rate is `{}` seconds.", channel.read().await.slow_mode_rate.unwrap_or(0))
    } else {
        "Failed to find channel in cache.".to_string()
    };

    if let Err(why) = msg.channel_id.say(&ctx.http, say_content).await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}
