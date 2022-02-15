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
  println!("[?] processing eval request from {}", msg.author.name);

  if msg.author.id.to_string() != env::var("DISCORD_OWNER_ID").expect("owner id") {
    println!("  [-] unauthorized request");

    msg
      .reply(ctx, "You don't have permission to do that.")
      .await?;

    return Ok(());
  } else if msg.content.is_empty() {
    msg.reply(ctx, "The message content is empty.").await?;

    return Ok(());
  }

  println!("  [+] request authorized; evaluating command");

  if !msg.content.starts_with(".eval\n```rs\n") || !msg.content.ends_with("\n```") {
    msg
      .reply(ctx, "The message content is not a valid code snippet. Ensure that the code block has the `rs` syntax format.")
      .await?;

    return Ok(());
  }

  let code_sample = msg.content.split("```rs\n").collect::<Vec<_>>()[1]
    .split("\n```")
    .collect::<Vec<_>>()[0];

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
    .arg(code_sample)
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

  if output.status.success() {
    msg
      .react(ctx, ReactionType::Unicode("✅".to_string()))
      .await?;
  } else {
    msg
      .react(ctx, ReactionType::Unicode("❌".to_string()))
      .await?;
  }

  msg
    .reply(
      ctx,
      format!("```llvm\n{}\n```", String::from_utf8_lossy(&output.stdout)),
    )
    .await?;

  Ok(())
}
