use core::str;
use std::{process::exit, vec};

use board::{encode_to_fen, gen_board};
use image::{GenericImage, ImageReader};
use serde::{Deserialize, Serialize};

mod board;

#[derive(Debug)]
enum PieceType {
	King,
	Queen,
	Rook,
	Bishop,
	Knight,
	Pawn,
}

impl PieceType {
	fn from_str(s: &str) -> Option<PieceType> {
		match s {
			"king" => Some(PieceType::King),
			"queen" => Some(PieceType::Queen),
			"rook" => Some(PieceType::Rook),
			"bishop" => Some(PieceType::Bishop),
			"knight" => Some(PieceType::Knight),
			"pawn" => Some(PieceType::Pawn),
			_ => None,
		}
	}
}

#[derive(Debug)]
enum Color {
	White,
	Black,
}

impl Color {
	fn from_str(s: &str) -> Option<Color> {
		match s {
			"white" => Some(Color::White),
			"black" => Some(Color::Black),
			_ => None,
		}
	}
}
#[derive(Debug, Serialize, Deserialize)]
// "{\"success\":true,\"evaluation\":-0.66,\"mate\":null,\"bestmove\":\"bestmove b8c6 ponder f1e2\",\"continuation\":\"b8c6 f1e2 d7d5 e4d5 f6d5 e1g1 f8d6 c2c4 d5e7 b1c3 e8g8 c1d2 f7f5 d1b3 f8f7 h2h3 h7h6 g1h1 f7f6 b3d1\"}
struct StockfishResponse {
	success: bool,
	evaluation: f32,
	mate: Option<i32>,
	bestmove: String,
	continuation: String,
}


#[derive(Debug)]
struct Piece {
	x: u8,
	y: u8,
	piece_type: PieceType,
	color: Color,
}


#[tokio::main]
async fn main() {
	let channel = "suba1805";
	let url = format!("https://lichess.org/@/{}/tv", channel);
	let mut board = [[0u8; 8]; 8];
	// set board at 3, 5 to 1

    let html = get_html(url).await.unwrap();
	let dom = tl::parse(html.as_str(), tl::ParserOptions::default()).unwrap();
	// get title
	let title = dom.query_selector("title").and_then(|mut iter| iter.next());
	if title.is_none() {
		println!("Title not found");
		exit(1);
	}
	let title = title.unwrap();
	println!("Title: {:?}", title.get(dom.parser()).unwrap().inner_text(dom.parser()));
	println!("Fetching Piece state for: {:?}", channel);
	let pieces = dom.query_selector("piece");

	if !pieces.is_some() {
		println!("Pieces not found");
		exit(1);
	}

	let orientation_b = dom.get_elements_by_class_name("player").next().unwrap();
	// let orientation_w = dom.get_elements_by_class_name("orientation-black").next();
	let classes = orientation_b.get(dom.parser()).unwrap().as_tag().unwrap().attributes().class().unwrap();

	// there should be 2
	let last_moves = dom.get_elements_by_class_name("last-move");
	// / vec of x and y


	let mut last_moves_x_y_vec: Vec<(f32, f32)> = vec![];

	for last_move in last_moves {
		let last_move = last_move.get(dom.parser()).unwrap().as_tag().unwrap();
		// as raw tag
		let last_move = last_move.attributes().get("style").unwrap().unwrap();
		let last_move: Vec<&str> = str::from_utf8(last_move.as_bytes()).unwrap().split_whitespace().collect();
		// top;left
		let last_move: Vec<_> = last_move[0].split(";").collect();
		let y: Vec<&str> = last_move[0].split(":").collect();
		let y: Vec<&str> = y[1].split("%").collect();
		let y = y[0];
		let y = y.parse::<f32>().unwrap();
		// to u8
		// let y: Vec<u8> = y[0].parse().unwrap();


		let x: Vec<_> = last_move[1].split(";").collect();
		let x: Vec<&str> = x[0].split(":").collect();
		let x: Vec<&str> = x[1].split("%").collect();
		let x = x[0];
		let x = x.parse::<f32>().unwrap();


		// println!("Last move: {:?} {:?}", x, y);
		// println!("Last move: {:?}, {:?}", x, y);
		last_moves_x_y_vec.push((x, y));
	}	



	let mode = match classes.as_utf8_str().split_whitespace().any(|x| x == "white") {
		// if white is first then it means our player is black
		true => "b",
		_ => "w",
	};
	// println!("Orientation W: {:?}", orientation_w);
	println!("Playing as: {:?}", match mode {
		"w" => "White",
		"b" => "Black",
		_ => "Unknown",
	});
	let pieces = pieces.unwrap();
	
	if pieces.clone().count() == 0 {
		println!("Not in an active game");
		exit(1);
	}

	for piece in pieces.into_iter() {
		let piece = piece.get(dom.parser()).unwrap().as_tag().unwrap();
		let class = piece.attributes().class().unwrap();

		// split class on whitespace, first is color and second is piece type
		let class: Vec<&str> = str::from_utf8(class.as_bytes()).unwrap().split_whitespace().collect();
		let color = Color::from_str(class[0]).unwrap();
		let piece_type = PieceType::from_str(class[1]).unwrap();
		let style = piece.attributes().get("style").unwrap().unwrap();
		// grab everythign inside "" 
		let style: Vec<&str> = str::from_utf8(style.as_bytes()).unwrap().split("\"").collect();
		let style = style[0];

		// split on ;
		let style: Vec<&str> = style.split(";").collect();
		// top = top:xx.x% and left = left:xx.x%
		// get the value of top and left
		let top: Vec<&str> = style[0].split(":").collect();
		let top = top[1];
		// remove % sign
		let top = top.replace("%", "");
		let top = top.parse::<f32>().unwrap();
		let top = board.len() as f32 * (top / 100.0);
		
		let left: Vec<&str> = style[1].split(":").collect();
		let left = left[1];
		let left = left.replace("%", "");
		let left = left.parse::<f32>().unwrap();
		let left = board[0].len() as f32 * (left / 100.0);

		// println!("top: {:?}, left: {:?}", top, left);

		let piece_type = match piece_type {
			PieceType::King => 1,
			PieceType::Queen => 2,
			PieceType::Rook => 3,
			PieceType::Bishop => 4,
			PieceType::Knight => 5,
			PieceType::Pawn => 6,
		};

		// replace with color
		let color = match color {
			Color::White => 1,
			Color::Black => 2,
		};

		let code = (color << 3) | piece_type;
		board[top as usize][left as usize] = code;
	}

	println!("\nBoard: ");
	for i in 0..8 {
		for j in 0..8 {

			let piece = board[i][j];
			if piece == 0 {
				print!("{:<2}, ", piece);
				continue;
			}

			let color = piece >> 3;
			let piece_type = piece & 0b111;

			let color = match color {
				1 => "W",
				2 => "B",
				_ => panic!("Invalid color {:?} at {:?}/{:?}, val {:?}", color, i, j, board[i][j]),
			};

			let piece_type = match piece_type {
				1 => "K",
				2 => "Q",
				3 => "R",
				4 => "B",
				5 => "N",
				6 => "P",
				_ => panic!("Invalid piece type"),
			};

			print!("{:<2}, ", format!("{}{}", color, piece_type));
		}
		println!();
	}

	// since we flip the board for stockfish we need to keep a copy of the original board
	let board_for_image = board.clone();

	// if mode is b then we need to flip the board
	if mode == "b" {
		let mut new_board = [[0u8; 8]; 8];
		for i in 0..8 {
			for j in 0..8 {
				new_board[i][j] = board[7 - i][7 - j];
			}
		}

		board = new_board;
	}

	println!();
	println!("Calculating FEN and sending to stockfish");
	// to fen
	
	let fen = encode_to_fen(board, mode);
	println!("FEN: {:?}", fen);

	println!();
	println!("fetching stockfish evaluation...");


	// https://stockfish.online/api/s/v2.php?fen=rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR%20w%20KQkq%20-%200%201&depth=15
	let url = format!("https://stockfish.online/api/s/v2.php?fen={}&depth=15", fen);
	let res = reqwest::get(url).await.unwrap();
	let body = res.text().await.unwrap();

	let stockfish: Result<StockfishResponse, serde_json::Error> = serde_json::from_str(&body);
	if stockfish.is_err() {
		println!("Error: {:?}", body);
		exit(1);
	}

	let stockfish = stockfish.unwrap();

	// best move

	// remove 'bestmove ' from bestmove
	let bestmove = stockfish.bestmove.replace("bestmove ", "");
	let ponder = bestmove.split_whitespace().nth(2).unwrap();
	let bestmove = bestmove.split_whitespace().nth(0).unwrap();
	let eval = stockfish.evaluation;


	let mut chance_to_win = (stockfish.evaluation) / 153.0 ;
	if mode == "b" {
		chance_to_win = 1.0 - chance_to_win;
	}

	let chance_to_win = (chance_to_win * 100.0) - 100.0;


	println!("Best Move: {:?}", bestmove);
	println!("Ponder: {:?}", ponder);
	println!("Evaluation: {:?}", stockfish.evaluation);
	if chance_to_win > 0.0 {
		println!("You are winning by {:.2}%.", chance_to_win);
	} else if chance_to_win < 40.0 {
		println!("Just quit.");
	}else if chance_to_win < 0.0 {
		println!("You are losing by {:.2}%.", chance_to_win);
	} else {
		println!("You are equal.");
	}
	println!("Mate: {:?}", stockfish.mate);
	println!("Continuation: {:?}", stockfish.continuation);

	gen_board(board_for_image, mode, last_moves_x_y_vec, bestmove.to_string());


}


pub async fn get_html(url: String) -> Result<String, reqwest::Error> {
	let res = reqwest::get(url).await?;
    Ok(res.text().await?)
}