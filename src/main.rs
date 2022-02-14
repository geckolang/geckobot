use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::model::prelude::ReactionType;
use std::process::Command;

use serenity::framework::standard::{
  macros::{command, group},
  CommandResult, StandardFramework,
};

use serenity::model::channel::Message;

use std::env;

#[group]
#[commands(eval)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
  println!("Bot init");

  let framework = StandardFramework::new()
    .configure(|c| c.prefix("."))
    .group(&GENERAL_GROUP);

  // Login with a bot token from the environment
  let token = env::var("DISCORD_TOKEN").expect("token");

  let mut client = Client::builder(token)
    .event_handler(Handler)
    .framework(framework)
    .await
    .expect("Error creating client");

  // start listening for events by starting a single shard
  if let Err(why) = client.start().await {
    println!("An error occurred while running the client: {:?}", why);
  }
}

#[command]
async fn eval(ctx: &Context, msg: &Message) -> CommandResult {
  println!("=> processing eval request from {}", msg.author.name);

  if msg.author.id.to_string() != env::var("DISCORD_OWNER_ID").expect("owner id") {
    msg
      .reply(ctx, "You don't have permission to do that.")
      .await?;

    return Ok(());
  } else if msg.content.is_empty() {
    msg.reply(ctx, "The message content is empty.").await?;

    return Ok(());
  }

  println!("=> command authorized; evaluating request");

  msg
    .react(
      ctx,
      ReactionType::Custom {
        animated: false,
        id: 939323497358958592.into(),
        name: Some("Working on it, boss!".to_string()),
      },
    )
    .await?;

  let command = Command::new("./eval.sh")
    .arg(msg.content.split(".eval").collect::<Vec<_>>()[1])
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn();

  if let Err(error) = command {
    msg
      .reply(ctx, format!("Eval script failed: {}", error))
      .await?;

    return Ok(());
  }

  let output = command
    .unwrap()
    .wait_with_output()
    .expect("failed to spawn eval script");

  let mut final_output_stream = &output.stdout;

  if output.status.success() {
    msg
      .react(ctx, ReactionType::Unicode("✅".to_string()))
      .await?;
  } else {
    msg
      .react(ctx, ReactionType::Unicode("❌".to_string()))
      .await?;

    final_output_stream = &output.stderr;
  }

  msg
    .reply(
      ctx,
      format!(
        "```llvm\n{}\n```",
        String::from_utf8_lossy(final_output_stream)
      ),
    )
    .await?;

  Ok(())
}
