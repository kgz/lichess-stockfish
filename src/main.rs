use core::str;
use std::sync::Arc;
use std::{env, process::exit, vec};

use board::{encode_to_fen, gen_board, help};
use image::{GenericImage, ImageReader};
use serde::{Deserialize, Serialize};
use serenity::all::standard::macros::hook;
use serenity::all::{ActivityData, Context, CreateAttachment, CreateButton, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateMessage, EditInteractionResponse, EditMessage, Emoji, EmojiIdentifier, EventHandler, GatewayIntents, Interaction, Message, MessageBuilder, PingInteraction};
use serenity::async_trait;
use serenity::client::Client;
use dotenv::dotenv;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;

mod board;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
		let args = msg.content.split_whitespace().collect::<Vec<&str>>();
        if args.len() == 1 && args[0].trim() == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }

		if args.len() == 2 && args[0] == "!help" {
			let typing = msg.channel_id.broadcast_typing(&ctx.http).await.unwrap();
			let channel = args[1];
			// to thread safe chennel
			let channel = channel;
			let channel = Arc::new(Mutex::new(&channel));
			let image_path = help(channel).await;
			println!("{}", image_path);
			// send image to discord
			let mut files: Vec<CreateAttachment> = vec![];
			let file = File::open(image_path.clone()).await.unwrap();
			files.push(CreateAttachment::file(&file, "test.png").await.unwrap());


			let test = MessageBuilder::new()
				.push("Here is the help image")
				.push_bold_line("test bold")
				.push_mono_line("test mono")
				
				.build();

			let embed = CreateEmbed::default()
				.title("Help")
				.description("This is the help image")
				.image("attachment://test.png")
				.attachment("test.png");

			let button = CreateButton::new("testButton")
				.label("test")
				.style(serenity::all::ButtonStyle::Primary)
				.custom_id("testButton")
				
				
				;		

			let message = CreateMessage::new()
				.content(test)
				.embed(embed)
				.button(button)
				
				;

		

			let ret = match msg.channel_id.send_files(&ctx.http, files, message).await {
				Ok(_) => (),
				Err(why) => println!("Error sending message: {why:?}")
			};

			// clean up image
			let _ = tokio::fs::remove_file(image_path).await;

			ret
			// async {
			// 	match msg.channel_id.say(&ctx.http, imageName).await {
			// 		Ok(_) => (),
			// 		Err(why) => println!("Error sending message: {why:?}")
			// 	}
			// }.await;
			
		}
    }

	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		println!("Interaction received");
		let t = interaction.id().to_string();
		
		println!("{:?}", t);
		ctx.set_activity(Some(ActivityData::custom("Getting the cookie")));
		println!("{:?}", t);		
		// let r = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(format!(
		// 	"Hello from interactions webhook HTTP server!",
		// )));

	
		let new_message = CreateInteractionResponseFollowup::new().content("Hello from interactions webhook HTTP server!");
		
		// update message it was called on
		let mut message = interaction.clone().message_component().unwrap();
		// message.edit_followup(cache_http, message_id, builder)

		// match interaction.as_message_component().unwrap().edit_followup(&ctx.http, message.message.id, new_message).await {
		// 	Ok(_) => (),
		// 	Err(why) => println!("Error sending message: {why:?}")
		// }

		let edit_message = EditMessage::new().content("Hello from interactions webhook HTTP server!").remove_all_attachments();

		match message.message.edit(&ctx.http, edit_message ).await {
			Ok(_) => (),
			Err(why) => println!("Error sending message: {why:?}")
		}

		let r = CreateInteractionResponse::Pong;
		
		match interaction.as_message_component().unwrap().create_response(&ctx.http, r).await {
			Ok(_) => (),
			Err(why) => println!("Error sending message: {why:?}")
		}

		
		// (&ctx.http, r).await {
		// 	Ok(_) => (),
		// 	Err(why) => println!("Error sending message: {why:?}")
		// }


		()
	}
	
}

#[hook]
async fn ready(ctx: Context, _data_about_bot: serenity::model::gateway::Ready) {
	println!("Bot is ready.");
	println!("{}", _data_about_bot.user.name);
	// print invite link
	println!("Invite link: {:?}", format!("https://discord.com/api/oauth2/authorize?client_id={}&permissions=8&scope=bot", _data_about_bot.user.id));
}

#[tokio::main]
async fn main() {
	dotenv().ok();
	let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

