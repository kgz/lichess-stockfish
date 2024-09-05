use core::str;
use std::sync::Arc;
use std::env;

use board::help;
use dotenv::dotenv;
use serenity::all::standard::macros::hook;
use serenity::all::{
    ChannelId, Context, CreateAttachment, CreateButton, CreateEmbed, CreateInteractionResponse, CreateMessage, EditMessage, EventHandler, GatewayIntents, Interaction, Message, MessageBuilder
};
use serenity::async_trait;
use serenity::client::Client;
use tokio::fs::File;
use tokio::sync::Mutex;

mod board;
mod schema;
mod models {
    pub mod message;
	pub mod error;
}

pub mod database {
    pub mod databse;
}

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
            let _ = msg.channel_id.broadcast_typing(&ctx.http).await.unwrap();
            let channel = args[1];
            // to thread safe chennel
            let og_channel = channel;
            let channel = Arc::new(Mutex::new(&channel));

			let mut description = MessageBuilder::new();

			description.push_bold("Evaluation: ");
			description.push_italic("Loading...");
			description.push("\n");
			
			description.push_bold("Forced Mate?: ");
			description.push_italic("Loading...");
			description.push("\n");
		
			description.push_bold("Best Move: ");
			description.push_italic("Loading...");
			description.push("\n");
			
			let description = description.build();

			let embed = CreateEmbed::default()
				.title(og_channel.to_uppercase())
				.description(description)
				.image("https://dummyimage.com/1024x1024/2b2d31/ffffff.png&text=Fetching+Stockfish...")
				;

			let button = CreateButton::new("testButton")
                .label("Refresh")
                .style(serenity::all::ButtonStyle::Primary)
                .custom_id("testButton")
				.disabled(true)
				;


			let message = CreateMessage::new()
				.embed(embed)
                .button(button)

				;

                
			let mut message = msg.channel_id.send_message(&ctx.http, message).await.unwrap();
			let _  = crate::models::message::Message::insert(
				crate::models::message::Message::new(og_channel.to_string(), message.id.to_string())
			);
			
            let stock_resp = help(channel).await;

			if stock_resp.is_err() {
				println!("Error getting help {:?}", stock_resp.as_ref().err().unwrap().to_string());
				// let _ = msg.channel_id.say(&ctx.http, format!("Error: {:?}", stock_resp.err())).await;
				let mut description = MessageBuilder::new();

				description.push_bold("Evaluation: ");
				description.push_italic("N/A");
				description.push("\n");
				
				description.push_bold("Forced Mate?: ");
				description.push_italic("N/A");
				description.push("\n");
			
				description.push_bold("Best Move: ");
				description.push_italic("N/A");
				description.push("\n");
				description.push("\n");
				description.push("\n");
				description.push_quote(format!("Error getting help {:?}", stock_resp.err().unwrap().to_string()));
				
				let description = description.build();
	
				let embed = CreateEmbed::default()
					.title(og_channel.to_uppercase())
					.description(description)
					;
	
				let button = CreateButton::new("testButton")
					.label("Refresh")
					.style(serenity::all::ButtonStyle::Danger)
					.custom_id("testButton")
					.disabled(true)
					
					;
	
	
				let new_message = EditMessage::new()
					.embed(embed)
					.button(button)
	
					;
				
				let _ = message.edit(&ctx.http, new_message).await.unwrap();
				return ();
			}
			let stock_resp = stock_resp.unwrap();


            println!("{}", stock_resp.clone().file);
            // send image to discord
            let mut files: Vec<CreateAttachment> = vec![];
            let file = File::open(format!("./pics/{}", stock_resp.clone().file)).await.unwrap();
            files.push(CreateAttachment::file(&file, "test.png").await.unwrap());

			let temp_message = CreateMessage::new();
			let _tchannel = ChannelId::new(167174376045805568 as u64);
			let chan = _tchannel.send_files(&ctx.http, files, temp_message).await;
	
			let attachment_url = chan.unwrap().attachments[0].url.clone();

			let mut description = MessageBuilder::new();

	
		
			let chance_to_win = stock_resp.evaluation;

			description.push_bold("Evaluation: ");
			if chance_to_win > 0.0 {
				description.push("You are winning by ");
				description.push(format!("{:.2}%", chance_to_win));
			} else if chance_to_win < -40.0 {
				description.push("Consider Conceeding. You are losing by ");
				description.push(format!("{:.2}%", chance_to_win));
			} else if chance_to_win < 0.0 {
				description.push("You are losing by ");
				description.push(format!("{:.2}%", chance_to_win));
			} else {
				println!("You are equal.");
			}
			description.push("\n");
			
			description.push_bold("Forced Mate?: ");
			if stock_resp.clone().mate.is_some() {
				description.push(stock_resp.clone().mate.unwrap().to_string());
				description.push("\n");
			} else {
				description.push("No");
				description.push("\n");
			}

			description.push_bold("Best Move: ");
			description.push(stock_resp.clone().bestmove.to_string());
			description.push("\n");
			
                
			let description = description.build();

			let button = CreateButton::new("testButton")
			.label("test")
			.style(serenity::all::ButtonStyle::Primary)
			.custom_id("testButton")
			.disabled(false)
			;

            let embed = CreateEmbed::default()
                .title(og_channel.to_uppercase())
                .description(description)
                .image(attachment_url);

       

            let message1 = EditMessage::new()
                .embed(embed)
				.button(button)
				;

			let _ = message.edit(&ctx.http, message1).await.unwrap();

            // clean up image
            let _ = tokio::fs::remove_file(stock_resp.clone().file).await;
            ()
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        println!("Interaction received");
        // ctx.set_activity(Some(ActivityData::custom("Getting the cookie")));

        // update message it was called on
        let referenced_message = interaction.clone().message_component().unwrap().message.id;
        let channel = crate::models::message::Message::find_by_channel_id(referenced_message.to_string());

		if channel.is_err() {
			println!("No referenced message {:?}", interaction.clone().message_component().unwrap().message);

			println!("No channel found");
			return ();
		}

		let channel = channel.unwrap();
		let og_channel = channel.clone().lc_channel;
		let channel = channel.lc_channel.as_str();

		let channel = Arc::new(Mutex::new(&channel));

        // send ping
		let r = CreateInteractionResponse::Acknowledge;
		let _ = interaction
            .as_message_component()
            .unwrap()
            .create_response(&ctx.http, r)
            .await;

		

		let mut description = MessageBuilder::new();

		description.push_bold("Evaluation: ");
		description.push_italic("Loading...");
		description.push("\n");
		
		description.push_bold("Forced Mate?: ");
		description.push_italic("Loading...");
		description.push("\n");
	
		description.push_bold("Best Move: ");
		description.push_italic("Loading...");
		description.push("\n");
		
		let description = description.build();

		let loading_embed = CreateEmbed::default()
			.description(description)
			.image("https://dummyimage.com/1024x1024/2b2d31/ffffff.png&text=Fetching+Stockfish...")
			;

		let loading_button = CreateButton::new("testButton")
			.style(serenity::all::ButtonStyle::Primary)
			.custom_id("testButton")
			.disabled(true)
			.label("Loading...")
			;


		let loading_message = EditMessage::new()
			.embed(loading_embed)
			.button(loading_button)

			;

		let _ = interaction.clone().message_component().unwrap().message.edit(&ctx.http, loading_message).await;
 




        let stock_resp = help(channel).await;

		if stock_resp.is_err() {
			let nmessage = EditMessage::new().content("Error getting help");
			let _ = interaction.clone().message_component().unwrap().message.edit(&ctx.http, nmessage).await;
			return ();
		}

		let stock_resp = stock_resp.unwrap();

	

		let mut files: Vec<CreateAttachment> = vec![];
        let file = File::open(format!("./pics/{}", stock_resp.clone().file)).await.unwrap();
		files.push(CreateAttachment::file(&file, stock_resp.clone().file).await.unwrap());
		


		let temp_message = CreateMessage::new();
		let _tchannel = ChannelId::new(167174376045805568 as u64);
		let chan = _tchannel.send_files(&ctx.http, files, temp_message).await;

		let attachment_url = chan.unwrap().attachments[0].url.clone();



		let mut description = MessageBuilder::new();

		let chance_to_win = stock_resp.evaluation;

		description.push_bold("Evaluation: ");
		if chance_to_win > 0.0 {
			description.push("You are winning by ");
			description.push(format!("{:.2}%", chance_to_win));
		} else if chance_to_win < -40.0 {
			description.push("Consider Conceeding. You are losing by ");
			description.push(format!("{:.2}%", chance_to_win));
		} else if chance_to_win < 0.0 {
			description.push("You are losing by ");
			description.push(format!("{:.2}%", chance_to_win));
		} else {
			description.push("equal");

			println!("You are equal.");
		}
			description.push("\n");
			
			description.push_bold("Forced Mate?: ");
			if stock_resp.clone().mate.is_some() {
				description.push(stock_resp.clone().mate.unwrap().to_string());
				description.push("\n");
			} else {
				description.push("No");
				description.push("\n");
			}

			description.push_bold("Best Move: ");
			description.push(stock_resp.clone().bestmove.to_string());
			description.push("\n");
			
                
			let description = description.build();

        let embed = CreateEmbed::default()
			.title(og_channel.to_uppercase())
			.description(description)
        	.image(attachment_url)
			// .attachment(image_path.clone())
		
			;

        let button = CreateButton::new("testButton")
        	.label("Refresh")
        	.style(serenity::all::ButtonStyle::Primary)
        	.custom_id("testButton")
			.disabled(false)
		;

	
		let mut message = interaction.clone().message_component().unwrap().message;
		
        let edit_message = EditMessage::new()
		.embed(embed.clone())
		.remove_all_attachments()
		.button(button)
		;



        match message.edit(&ctx.http, edit_message).await {
            Ok(_) => (),
            Err(why) => println!("Error sending message: {why:?}"),
        }
        ()
    }
}

#[hook]
async fn ready(_: Context, _data_about_bot: serenity::model::gateway::Ready) {
    println!("Bot is ready.");
    println!("{}", _data_about_bot.user.name);
    // print invite link
    println!(
        "Invite link: {:?}",
        format!(
            "https://discord.com/api/oauth2/authorize?client_id={}&permissions=8&scope=bot",
            _data_about_bot.user.id
        )
    );
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
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
